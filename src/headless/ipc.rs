//! Local IPC between the CLI client and the headless player over a Windows named
//! pipe. Messages are length-prefixed JSON frames (4-byte little-endian length
//! followed by the body), kept symmetric so both ends share the same codec.
//!
//! The pipe lives in the default local namespace, so only processes on the same
//! machine can reach it. Frame sizes are capped to reject malformed input.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};

pub const PIPE_NAME: &str = r"\\.\pipe\reverbic-control";

const MAX_FRAME_LEN: u32 = 64 * 1024;

#[derive(Serialize, Deserialize)]
pub enum Request {
    Play { query: Option<String> },
    Stop,
    Status,
    Volume(u8),
    Toggle,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Message(String),
    Status(StatusReport),
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct StatusReport {
    pub state: String,
    pub station: Option<String>,
    pub title: Option<String>,
    pub volume: u8,
}

/// Single-instance server endpoint. Binding fails if another player already
/// holds the pipe, which is exactly how we enforce one daemon per machine.
pub struct Server {
    instance: NamedPipeServer,
}

impl Server {
    pub fn bind() -> io::Result<Self> {
        let instance = ServerOptions::new()
            .first_pipe_instance(true)
            .create(PIPE_NAME)?;
        Ok(Self { instance })
    }

    /// Waits for the next client and returns its connection, immediately
    /// re-arming a fresh instance so subsequent clients are never refused.
    pub async fn accept(&mut self) -> io::Result<NamedPipeServer> {
        self.instance.connect().await?;
        let next = ServerOptions::new().create(PIPE_NAME)?;
        Ok(std::mem::replace(&mut self.instance, next))
    }
}

pub fn try_connect() -> Option<NamedPipeClient> {
    ClientOptions::new().open(PIPE_NAME).ok()
}

pub async fn write_frame<W, T>(writer: &mut W, value: &T) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let body =
        serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = body.len() as u32;
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds maximum size",
        ));
    }
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(&body).await?;
    writer.flush().await
}

pub async fn read_frame<R, T>(reader: &mut R) -> io::Result<T>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf);
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds maximum size",
        ));
    }
    let mut body = vec![0u8; len as usize];
    reader.read_exact(&mut body).await?;
    serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
