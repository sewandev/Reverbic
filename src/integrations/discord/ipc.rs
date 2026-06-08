#![cfg(target_os = "windows")]

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::ClientOptions;

const PIPE_NAMES: &[&str] = &[
    r"\\.\pipe\discord-ipc-0",
    r"\\.\pipe\discord-ipc-1",
    r"\\.\pipe\discord-ipc-2",
    r"\\.\pipe\discord-ipc-3",
    r"\\.\pipe\discord-ipc-4",
    r"\\.\pipe\discord-ipc-5",
    r"\\.\pipe\discord-ipc-6",
    r"\\.\pipe\discord-ipc-7",
    r"\\.\pipe\discord-ipc-8",
    r"\\.\pipe\discord-ipc-9",
];

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct DiscordIpc {
    pipe: tokio::net::windows::named_pipe::NamedPipeClient,
}

impl DiscordIpc {
    pub fn connect() -> Option<Self> {
        for name in PIPE_NAMES {
            if let Ok(pipe) = ClientOptions::new().open(name) {
                return Some(Self { pipe });
            }
        }
        None
    }

    pub async fn handshake(&mut self, client_id: &str) -> bool {
        let payload = format!(r#"{{"v":1,"client_id":"{}"}}"#, client_id);
        if self.send_frame(OP_HANDSHAKE, &payload).await.is_err() {
            return false;
        }
        let mut header = [0u8; 8];
        let read_result =
            tokio::time::timeout(HANDSHAKE_TIMEOUT, self.pipe.read_exact(&mut header)).await;
        let Ok(Ok(_)) = read_result else {
            return false;
        };
        let len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let mut body = vec![0u8; len];
        let _ = tokio::time::timeout(HANDSHAKE_TIMEOUT, self.pipe.read_exact(&mut body)).await;
        true
    }

    pub async fn set_activity(&mut self, pid: u32, activity_json: &str) -> bool {
        let payload = format!(
            r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":{}}},"nonce":"{}"}}"#,
            pid,
            activity_json,
            nonce()
        );
        self.send_frame(OP_FRAME, &payload).await.is_ok()
    }

    pub async fn clear_activity(&mut self, pid: u32) -> bool {
        let payload = format!(
            r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":null}},"nonce":"{}"}}"#,
            pid,
            nonce()
        );
        self.send_frame(OP_FRAME, &payload).await.is_ok()
    }

    async fn send_frame(&mut self, opcode: u32, data: &str) -> std::io::Result<()> {
        let bytes = data.as_bytes();
        let len = bytes.len() as u32;
        let mut frame = Vec::with_capacity(8 + bytes.len());
        frame.extend_from_slice(&opcode.to_le_bytes());
        frame.extend_from_slice(&len.to_le_bytes());
        frame.extend_from_slice(bytes);
        self.pipe.write_all(&frame).await
    }
}

fn nonce() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
