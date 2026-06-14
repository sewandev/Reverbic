//! The background player process. It owns the audio pipeline and serves control
//! requests over the named pipe until told to stop. No terminal UI is created.

use crate::audio::{AudioPlayer, PlayerCommand, PlayerStatus};
use crate::config::{Config, LastStation};
use crate::headless::ipc::{self, Request, Response, StatusReport};
use crate::headless::resolve;

pub async fn run() -> crate::error::Result<()> {
    let mut config = Config::load();
    crate::i18n::init(config.language);

    let mut server = match ipc::Server::bind() {
        Ok(server) => server,
        Err(e) => {
            tracing::info!("Another reverbic player already owns the control pipe: {e}");
            return Ok(());
        }
    };

    let player = AudioPlayer::spawn();
    if config.restore_volume {
        player.send(PlayerCommand::SetVolume(config.volume)).await;
    }
    tracing::info!("headless player started");

    loop {
        let mut connection = match server.accept().await {
            Ok(connection) => connection,
            Err(e) => {
                tracing::warn!("failed to accept control connection: {e}");
                continue;
            }
        };

        let request = match ipc::read_frame::<_, Request>(&mut connection).await {
            Ok(request) => request,
            Err(e) => {
                tracing::warn!("malformed control request: {e}");
                continue;
            }
        };

        let (response, shutdown) = handle(&player, &mut config, request).await;
        if let Err(e) = ipc::write_frame(&mut connection, &response).await {
            tracing::warn!("failed to answer control request: {e}");
        }
        if shutdown {
            tracing::info!("headless player shutting down");
            return Ok(());
        }
    }
}

async fn handle(player: &AudioPlayer, config: &mut Config, request: Request) -> (Response, bool) {
    match request {
        Request::Play { query } => match resolve::resolve(query.as_deref(), config).await {
            Ok(station) => {
                config.last_station = Some(LastStation::from_station(&station));
                config.save();
                let name = station.name.clone();
                player.send(PlayerCommand::Play(station)).await;
                (Response::Message(format!("Playing {name}")), false)
            }
            Err(message) => (Response::Error(message), false),
        },
        Request::Stop => {
            player.send(PlayerCommand::Stop).await;
            (Response::Message("Stopped".to_string()), true)
        }
        Request::Status => (
            Response::Status(status_report(player, config.volume)),
            false,
        ),
        Request::Volume(level) => {
            let volume = level as f32 / 100.0;
            player.send(PlayerCommand::SetVolume(volume)).await;
            config.volume = volume;
            config.save();
            (Response::Message(format!("Volume set to {level}%")), false)
        }
        Request::Toggle => toggle(player).await,
    }
}

async fn toggle(player: &AudioPlayer) -> (Response, bool) {
    match player.state().status {
        PlayerStatus::Playing
        | PlayerStatus::Buffering(_)
        | PlayerStatus::Connecting
        | PlayerStatus::Reconnecting(_) => {
            player.send(PlayerCommand::Pause).await;
            (Response::Message("Paused".to_string()), false)
        }
        PlayerStatus::Paused => {
            player.send(PlayerCommand::Resume).await;
            (Response::Message("Resumed".to_string()), false)
        }
        PlayerStatus::Idle | PlayerStatus::Error(_) => {
            (Response::Error("Nothing is playing.".to_string()), false)
        }
    }
}

fn status_report(player: &AudioPlayer, volume: f32) -> StatusReport {
    let state = player.state();
    StatusReport {
        state: status_label(&state.status),
        station: state.station.map(|station| station.name),
        title: state.title,
        volume: (volume * 100.0).round() as u8,
    }
}

fn status_label(status: &PlayerStatus) -> String {
    match status {
        PlayerStatus::Idle => "Idle",
        PlayerStatus::Connecting => "Connecting",
        PlayerStatus::Buffering(_) => "Buffering",
        PlayerStatus::Reconnecting(_) => "Reconnecting",
        PlayerStatus::Playing => "Playing",
        PlayerStatus::Paused => "Paused",
        PlayerStatus::Error(_) => "Error",
    }
    .to_string()
}
