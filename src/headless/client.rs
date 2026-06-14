//! The foreground CLI. It resolves the requested action into an IPC request,
//! talks to the running daemon (spawning a detached one when needed) and prints
//! the outcome before returning control of the shell.

use std::os::windows::process::CommandExt;
use std::process::{Command as ProcessCommand, Stdio};
use std::time::Duration;

use tokio::net::windows::named_pipe::NamedPipeClient;

use crate::headless::ipc::{self, Request, Response, StatusReport};
use crate::headless::Command;

const DETACHED_PROCESS: u32 = 0x0000_0008;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const SPAWN_RETRIES: u32 = 50;
const SPAWN_RETRY_DELAY: Duration = Duration::from_millis(100);

pub async fn run(command: Command) -> crate::error::Result<()> {
    match command {
        Command::Play { query } => {
            let request = Request::Play {
                query: join_query(query),
            };
            match connect(true).await {
                Some(client) => print_response(exchange(client, &request).await),
                None => eprintln!("Could not start the background player."),
            }
        }
        Command::Stop => dispatch(Request::Stop, "Nothing is playing.").await,
        Command::Status => {
            dispatch(Request::Status, "Stopped (no background player running).").await
        }
        Command::Volume { level } => dispatch(Request::Volume(level), "Nothing is playing.").await,
        Command::Toggle => dispatch(Request::Toggle, "Nothing is playing.").await,
        Command::Daemon => unreachable!("daemon is handled before reaching the client"),
    }
    Ok(())
}

/// Sends a request that requires an already-running daemon, printing
/// `absent_message` when none is reachable.
async fn dispatch(request: Request, absent_message: &str) {
    match connect(false).await {
        Some(client) => print_response(exchange(client, &request).await),
        None => println!("{absent_message}"),
    }
}

async fn exchange(mut client: NamedPipeClient, request: &Request) -> Response {
    if let Err(e) = ipc::write_frame(&mut client, request).await {
        return Response::Error(format!("Failed to send command: {e}"));
    }
    match ipc::read_frame::<_, Response>(&mut client).await {
        Ok(response) => response,
        Err(e) => Response::Error(format!("No reply from the background player: {e}")),
    }
}

/// Connects to the daemon, optionally spawning a detached one and waiting for it
/// to come up. Returns `None` when no daemon is (or becomes) reachable.
async fn connect(spawn_if_absent: bool) -> Option<NamedPipeClient> {
    if let Some(client) = ipc::try_connect() {
        return Some(client);
    }
    if !spawn_if_absent || spawn_daemon().is_err() {
        return None;
    }
    for _ in 0..SPAWN_RETRIES {
        tokio::time::sleep(SPAWN_RETRY_DELAY).await;
        if let Some(client) = ipc::try_connect() {
            return Some(client);
        }
    }
    None
}

fn spawn_daemon() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    ProcessCommand::new(exe)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP)
        .spawn()?;
    Ok(())
}

fn join_query(parts: Vec<String>) -> Option<String> {
    let query = parts.join(" ");
    let query = query.trim();
    if query.is_empty() {
        None
    } else {
        Some(query.to_string())
    }
}

fn print_response(response: Response) {
    match response {
        Response::Message(message) => println!("{message}"),
        Response::Status(report) => print_status(report),
        Response::Error(message) => eprintln!("{message}"),
    }
}

fn print_status(report: StatusReport) {
    println!("State:   {}", report.state);
    if let Some(station) = report.station {
        println!("Station: {station}");
    }
    if let Some(title) = report.title {
        println!("Title:   {title}");
    }
    println!("Volume:  {}%", report.volume);
}
