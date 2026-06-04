use bytes::Bytes;
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::{mpsc, Mutex};

use crate::metadata::parse_icy_title;

pub struct StreamReader {
    rx: Mutex<mpsc::Receiver<Bytes>>,
    chunks: VecDeque<Bytes>,
    offset: usize,
    buffered: usize,
    last_chunk_at: Arc<AtomicU64>,
    download_done: Arc<AtomicBool>,
    dead_url: Arc<AtomicBool>,
}

impl StreamReader {
    fn push_chunk(&mut self, chunk: Bytes) {
        if chunk.is_empty() {
            return;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.last_chunk_at.store(now, Ordering::Release);
        self.buffered += chunk.len();
        self.chunks.push_back(chunk);
    }
    pub fn last_chunk_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.last_chunk_at)
    }

    pub fn connect(
        url: String,
        start_byte: u64,
        channel_size: usize,
        handle: tokio::runtime::Handle,
    ) -> (Self, mpsc::Receiver<String>) {
        let (audio_tx, audio_rx) = mpsc::sync_channel::<Bytes>(channel_size);
        let (title_tx, title_rx) = mpsc::sync_channel::<String>(8);

        let download_done = Arc::new(AtomicBool::new(false));
        let done_for_task = Arc::clone(&download_done);
        let dead_url = Arc::new(AtomicBool::new(false));
        let dead_for_task = Arc::clone(&dead_url);

        handle.spawn(async move {
            if let Err(e) = download_stream(
                url,
                start_byte,
                audio_tx,
                title_tx,
                done_for_task,
                dead_for_task,
            )
            .await
            {
                tracing::error!("Stream download failed: {e}");
            }
        });

        let last_chunk_at = Arc::new(AtomicU64::new(0));
        let reader = Self {
            rx: Mutex::new(audio_rx),
            chunks: VecDeque::new(),
            offset: 0,
            buffered: 0,
            last_chunk_at,
            download_done,
            dead_url,
        };
        (reader, title_rx)
    }

    pub fn download_done_arc(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.download_done)
    }

    pub fn dead_url_arc(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.dead_url)
    }

    pub fn connect_preview(url: String, handle: tokio::runtime::Handle) -> Self {
        let (audio_tx, audio_rx) = mpsc::sync_channel::<Bytes>(64);

        handle.spawn(async move {
            if let Err(e) = download_preview(url, audio_tx).await {
                tracing::error!("Preview download failed: {e}");
            }
        });

        let last_chunk_at = Arc::new(AtomicU64::new(0));
        Self {
            rx: Mutex::new(audio_rx),
            chunks: VecDeque::new(),
            offset: 0,
            buffered: 0,
            last_chunk_at,
            download_done: Arc::new(AtomicBool::new(false)),
            dead_url: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn pre_buffer(&mut self, target_bytes: usize, mut progress: impl FnMut(f32)) {
        while self.buffered < target_bytes {
            let rx = self.rx.lock().expect("StreamReader mutex poisoned");
            let result = rx.recv_timeout(std::time::Duration::from_millis(100));
            drop(rx);
            match result {
                Ok(chunk) => {
                    self.push_chunk(chunk);
                    progress((self.buffered as f32 / target_bytes as f32).clamp(0.0, 1.0));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    progress((self.buffered as f32 / target_bytes as f32).clamp(0.0, 1.0));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        if self.buffered == 0 {
            let rx = self.rx.lock().expect("StreamReader mutex poisoned");
            match rx.recv() {
                Ok(chunk) => {
                    drop(rx);
                    self.push_chunk(chunk);
                }
                Err(_) => return Ok(0),
            }
        }
        {
            let mut pending: Vec<Bytes> = Vec::new();
            {
                let rx = self.rx.lock().expect("StreamReader mutex poisoned");
                while let Ok(chunk) = rx.try_recv() {
                    pending.push(chunk);
                }
            }
            for chunk in pending {
                self.push_chunk(chunk);
            }
        }
        let mut written = 0;
        while written < out.len() {
            let front = match self.chunks.front() {
                Some(c) => c,
                None => break,
            };
            let available = front.len() - self.offset;
            let take = available.min(out.len() - written);
            out[written..written + take].copy_from_slice(&front[self.offset..self.offset + take]);
            written += take;
            self.offset += take;
            self.buffered -= take;
            if self.offset >= front.len() {
                self.chunks.pop_front();
                self.offset = 0;
            }
        }
        Ok(written)
    }
}

impl Seek for StreamReader {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "stream is not seekable",
        ))
    }
}

async fn download_stream(
    url: String,
    start_byte: u64,
    audio_tx: mpsc::SyncSender<Bytes>,
    title_tx: mpsc::SyncSender<String>,
    download_done: Arc<AtomicBool>,
    dead_url: Arc<AtomicBool>,
) -> Result<(), reqwest::Error> {
    tracing::info!("Conectando a stream: {url} (start_byte={start_byte})");

    let mut req_headers = reqwest::header::HeaderMap::new();
    req_headers.insert(
        "Icy-MetaData",
        reqwest::header::HeaderValue::from_static("1"),
    );
    req_headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("reverbic/0.1"),
    );
    if start_byte > 0 {
        let range_val = format!("bytes={start_byte}-");
        if let Ok(v) = reqwest::header::HeaderValue::from_str(&range_val) {
            req_headers.insert(reqwest::header::RANGE, v);
        }
    }

    let client = reqwest::Client::builder()
        .default_headers(req_headers)
        .connect_timeout(std::time::Duration::from_secs(5))
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .build()?;

    let resp = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get(&url).send(),
    )
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            tracing::warn!("Timeout al conectar a {url} (5s sin respuesta)");
            return Ok(());
        }
    };

    log_response_diagnostics(&url, &resp);

    let status = resp.status();
    if !status.is_success() {
        tracing::error!("HTTP error {status} para URL: {url}");
        if status == reqwest::StatusCode::NOT_FOUND {
            dead_url.store(true, Ordering::Release);
        }
        return Ok(());
    }

    let metaint = parse_icy_metaint(&resp);
    tracing::info!("icy-metaint={metaint}");

    let mut stripper = IcyStripper::new(metaint, title_tx);
    let mut stream = resp.bytes_stream();
    let mut total_audio_bytes: usize = 0;
    let mut first_chunk_logged = false;

    use futures_util::StreamExt;
    while let Ok(Some(chunk)) =
        tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await
    {
        match chunk {
            Ok(raw_bytes) => {
                if !first_chunk_logged {
                    log_first_chunk(&raw_bytes);
                    first_chunk_logged = true;
                }

                let mut audio = Vec::with_capacity(raw_bytes.len());
                stripper.process(&raw_bytes, &mut audio);

                if audio.is_empty() {
                    continue;
                }

                total_audio_bytes += audio.len();

                if audio_tx.send(Bytes::from(audio)).is_err() {
                    tracing::debug!("Receiver cerrado ({total_audio_bytes} bytes enviados)");
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Error en chunk ({total_audio_bytes} bytes recibidos): {e}");
            }
        }
    }

    tracing::info!("Stream terminado: {total_audio_bytes} bytes de audio totales");
    download_done.store(true, Ordering::Release);
    Ok(())
}

async fn download_preview(
    url: String,
    audio_tx: mpsc::SyncSender<Bytes>,
) -> Result<(), reqwest::Error> {
    tracing::info!("Preview: iniciando descarga desde {url}");

    let client = reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;

    let resp = client.get(&url).send().await?;
    let status = resp.status();
    let content_length = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("desconocido")
        .to_string();
    let transfer_encoding = resp
        .headers()
        .get("transfer-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none")
        .to_string();

    tracing::info!(
        "Preview: HTTP {status} | content-type={content_type} \
         | content-length={content_length:?} | transfer-encoding={transfer_encoding}"
    );

    if !status.is_success() {
        tracing::warn!("Preview: HTTP {status} — abortando descarga");
        return Ok(());
    }

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut skip_remaining: usize = 0;
    let mut header_buf: Vec<u8> = Vec::new();
    let mut header_analyzed = false;
    let mut total_raw: usize = 0;
    let mut total_audio: usize = 0;
    let mut chunk_count: usize = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(
                    "Preview: error en chunk #{chunk_count} \
                     (raw={total_raw} audio={total_audio} esperado={content_length:?}): {e}"
                );
                break;
            }
        };

        chunk_count += 1;
        total_raw += chunk.len();
        tracing::debug!(
            "Preview: chunk #{chunk_count} {len}B (raw_total={total_raw} audio_total={total_audio})",
            len = chunk.len()
        );

        let to_send: Bytes = if !header_analyzed {
            header_buf.extend_from_slice(&chunk);
            if header_buf.len() < 10 {
                continue;
            }

            if header_buf[..3] == *b"ID3" {
                let id3_body = ((header_buf[6] as usize) << 21)
                    | ((header_buf[7] as usize) << 14)
                    | ((header_buf[8] as usize) << 7)
                    | (header_buf[9] as usize);
                skip_remaining = 10 + id3_body;
                tracing::info!(
                    "Preview: ID3v2 detectado — tag={skip_remaining}B \
                     ({:.1}% del archivo estimado)",
                    content_length
                        .map(|cl| skip_remaining as f32 / cl as f32 * 100.0)
                        .unwrap_or(0.0)
                );
            } else {
                tracing::debug!("Preview: sin ID3v2 — audio comienza en byte 0");
            }
            header_analyzed = true;

            let buf = std::mem::take(&mut header_buf);
            if skip_remaining >= buf.len() {
                skip_remaining -= buf.len();
                continue;
            }
            let audio_start = skip_remaining;
            skip_remaining = 0;
            Bytes::from(buf[audio_start..].to_vec())
        } else if skip_remaining > 0 {
            if skip_remaining >= chunk.len() {
                skip_remaining -= chunk.len();
                continue;
            }
            let start = skip_remaining;
            skip_remaining = 0;
            chunk.slice(start..)
        } else {
            chunk
        };

        if to_send.is_empty() {
            continue;
        }

        total_audio += to_send.len();
        if audio_tx.send(to_send).is_err() {
            tracing::info!(
                "Preview: decoder cerró el canal — \
                 raw={total_raw} audio={total_audio} chunks={chunk_count}"
            );
            return Ok(());
        }
    }

    match content_length {
        Some(expected) if total_raw < expected => {
            tracing::warn!(
                "Preview: descarga TRUNCADA — recibido={total_raw}B esperado={expected}B \
                 diferencia={}B | audio_enviado={total_audio}B chunks={chunk_count}",
                expected.saturating_sub(total_raw)
            );
        }
        Some(expected) => {
            tracing::info!(
                "Preview: descarga completa — raw={total_raw}B/{expected}B \
                 audio={total_audio}B chunks={chunk_count}"
            );
        }
        None => {
            tracing::info!(
                "Preview: descarga completa (sin Content-Length) — \
                 raw={total_raw}B audio={total_audio}B chunks={chunk_count}"
            );
        }
    }

    Ok(())
}

fn parse_icy_metaint(resp: &reqwest::Response) -> usize {
    resp.headers()
        .get("icy-metaint")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn log_response_diagnostics(url: &str, resp: &reqwest::Response) {
    tracing::info!(
        url        = url,
        status     = %resp.status(),
        content_type = ?resp.headers().get("content-type"),
        icy_name   = ?resp.headers().get("icy-name"),
        icy_genre  = ?resp.headers().get("icy-genre"),
        icy_br     = ?resp.headers().get("icy-br"),
        icy_metaint = ?resp.headers().get("icy-metaint"),
        "Respuesta HTTP recibida"
    );
}

fn log_first_chunk(bytes: &Bytes) {
    let preview = &bytes[..bytes.len().min(16)];
    match std::str::from_utf8(&bytes[..bytes.len().min(64)]) {
        Ok(text) => tracing::warn!(
            first_bytes_utf8 = text,
            total_chunk_len = bytes.len(),
            "Primer chunk es texto — posible respuesta HTML"
        ),
        Err(_) => tracing::info!(
            first_bytes_hex = %preview.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "),
            total_chunk_len = bytes.len(),
            "Primer chunk es binario — parece audio"
        ),
    }
}

#[derive(Clone, Copy)]
enum IcyState {
    Audio(usize),
    MetaLen,
    MetaData(usize),
}

struct IcyStripper {
    state: IcyState,
    metaint: usize,
    title_tx: mpsc::SyncSender<String>,
    meta_buf: Vec<u8>,
}

impl IcyStripper {
    fn new(metaint: usize, title_tx: mpsc::SyncSender<String>) -> Self {
        Self {
            state: if metaint > 0 {
                IcyState::Audio(metaint)
            } else {
                IcyState::Audio(usize::MAX)
            },
            metaint,
            title_tx,
            meta_buf: Vec::new(),
        }
    }

    fn process(&mut self, mut input: &[u8], output: &mut Vec<u8>) {
        while !input.is_empty() {
            match self.state {
                IcyState::Audio(remaining) => {
                    if self.metaint == 0 {
                        output.extend_from_slice(input);
                        return;
                    }
                    let take = remaining.min(input.len());
                    output.extend_from_slice(&input[..take]);
                    input = &input[take..];
                    self.state = if remaining == take {
                        IcyState::MetaLen
                    } else {
                        IcyState::Audio(remaining - take)
                    };
                }
                IcyState::MetaLen => {
                    let meta_bytes = input[0] as usize * 16;
                    input = &input[1..];
                    self.state = if meta_bytes == 0 {
                        IcyState::Audio(self.metaint)
                    } else {
                        self.meta_buf.clear();
                        self.meta_buf.reserve(meta_bytes);
                        IcyState::MetaData(meta_bytes)
                    };
                }
                IcyState::MetaData(remaining) => {
                    let take = remaining.min(input.len());
                    self.meta_buf.extend_from_slice(&input[..take]);
                    input = &input[take..];
                    self.state = if remaining == take {
                        self.emit_title();
                        IcyState::Audio(self.metaint)
                    } else {
                        IcyState::MetaData(remaining - take)
                    };
                }
            }
        }
    }

    fn emit_title(&self) {
        let raw = String::from_utf8_lossy(&self.meta_buf);
        let raw = raw.trim_end_matches('\0');
        if let Some(title) = parse_icy_title(raw) {
            tracing::debug!("ICY title: {title}");
            let _ = self.title_tx.try_send(title);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn make_stripper(metaint: usize) -> (IcyStripper, mpsc::Receiver<String>) {
        let (tx, rx) = mpsc::sync_channel(8);
        (IcyStripper::new(metaint, tx), rx)
    }

    #[test]
    fn passthrough_when_no_metaint() {
        let (mut s, rx) = make_stripper(0);
        let input = b"hello audio bytes";
        let mut out = Vec::new();
        s.process(input, &mut out);
        assert_eq!(out, input);
        assert!(rx.try_recv().is_err(), "no title expected");
    }

    #[test]
    fn strips_metadata_block() {
        let metaint = 8usize;
        let (mut s, rx) = make_stripper(metaint);
        let title_str = b"StreamTitle='Test Artist - Test Track';StreamUrl='';";
        let padded_len = title_str.len().div_ceil(16) * 16;
        let mut meta_block = vec![0u8; padded_len];
        meta_block[..title_str.len()].copy_from_slice(title_str);
        let meta_len_byte = (padded_len / 16) as u8;

        let audio = b"AUDIOBYT";
        let mut input = Vec::new();
        input.extend_from_slice(audio);
        input.push(meta_len_byte);
        input.extend_from_slice(&meta_block);

        let mut out = Vec::new();
        s.process(&input, &mut out);
        assert_eq!(out, audio);
        let title = rx.try_recv().expect("debería haber un título");
        assert_eq!(title, "Test Artist - Test Track");
    }

    #[test]
    fn empty_metadata_block() {
        let metaint = 4usize;
        let (mut s, rx) = make_stripper(metaint);
        let input = vec![1u8, 2, 3, 4, 0u8, 5, 6, 7, 8];
        let mut out = Vec::new();
        s.process(&input, &mut out);

        assert_eq!(out, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(rx.try_recv().is_err(), "no title expected con meta_len=0");
    }

    #[test]
    fn state_survives_split_chunks() {
        let metaint = 4usize;
        let (mut s, rx) = make_stripper(metaint);

        let title_str = b"StreamTitle='Chunked';StreamUrl='';";
        let padded_len = title_str.len().div_ceil(16) * 16;
        let mut meta_block = vec![0u8; padded_len];
        meta_block[..title_str.len()].copy_from_slice(title_str);
        let meta_len_byte = (padded_len / 16) as u8;
        let mut out = Vec::new();
        s.process(&[10, 20], &mut out);
        assert_eq!(out, [10, 20]);
        out.clear();
        s.process(&[30, 40, meta_len_byte], &mut out);
        assert_eq!(out, [30, 40]);
        out.clear();
        let mid = meta_block.len() / 2;
        s.process(&meta_block[..mid], &mut out);
        assert!(out.is_empty());

        s.process(&meta_block[mid..], &mut out);
        assert!(out.is_empty());

        let title = rx
            .try_recv()
            .expect("título emitido tras completar el bloque");
        assert_eq!(title, "Chunked");
    }
}
