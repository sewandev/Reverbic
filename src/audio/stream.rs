use bytes::Bytes;
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
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
        custom_headers: Option<HashMap<String, String>>,
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
                custom_headers,
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
            let rx = self.rx.lock().unwrap_or_else(|e| e.into_inner());
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
            let rx = self.rx.lock().unwrap_or_else(|e| e.into_inner());
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
                let rx = self.rx.lock().unwrap_or_else(|e| e.into_inner());
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

const MAX_DOWNLOAD_RETRIES: u32 = 5;
const DOWNLOAD_RETRY_PAUSE: std::time::Duration = std::time::Duration::from_secs(2);
const FILE_READ_STARVATION_SECS: u64 = 60;

pub fn youtube_cache_dir() -> std::path::PathBuf {
    crate::paths::youtube_media_cache_dir()
}

pub fn clear_youtube_cache() {
    let dir = youtube_cache_dir();
    if dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            tracing::debug!("could not clear youtube cache dir: {e}");
        }
    }
}

pub struct FileBackedReader {
    file: std::fs::File,
    pos: u64,
    written: Arc<AtomicU64>,
    total_len: Arc<AtomicU64>,
    last_chunk_at: Arc<AtomicU64>,
    download_done: Arc<AtomicBool>,
    dead_url: Arc<AtomicBool>,
}

pub struct FileBackedDownload {
    task: tokio::task::JoinHandle<()>,
    path: PathBuf,
    download_done: Arc<AtomicBool>,
}

impl FileBackedDownload {
    pub fn cancel(self, handle: &tokio::runtime::Handle) -> Option<PathBuf> {
        let Self {
            task,
            path,
            download_done,
        } = self;
        let remove_partial = !download_done.load(Ordering::Acquire);
        task.abort();
        handle.block_on(async {
            match task.await {
                Ok(()) => {}
                Err(e) if e.is_cancelled() => {
                    tracing::debug!("File-backed download task cancelled");
                }
                Err(e) => {
                    tracing::warn!("File-backed download task ended with error: {e}");
                }
            }
        });
        remove_partial.then_some(path)
    }

    pub fn cancel_and_cleanup(self, handle: &tokio::runtime::Handle) {
        if let Some(path) = self.cancel(handle) {
            remove_partial_file(&path);
        }
    }

    pub fn cleanup_partial(path: PathBuf) {
        remove_partial_file(&path);
    }

    #[cfg(test)]
    pub(crate) fn from_parts_for_test(
        task: tokio::task::JoinHandle<()>,
        path: PathBuf,
        download_done: Arc<AtomicBool>,
    ) -> Self {
        Self {
            task,
            path,
            download_done,
        }
    }
}

fn remove_partial_file(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => tracing::debug!(path = %path.display(), "removed partial file-backed download"),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => tracing::debug!(
            path = %path.display(),
            "could not remove partial file-backed download: {e}"
        ),
    }
}

impl FileBackedReader {
    pub fn create(
        url: String,
        custom_headers: Option<HashMap<String, String>>,
        path: PathBuf,
        handle: tokio::runtime::Handle,
    ) -> io::Result<(Self, FileBackedDownload, mpsc::Receiver<String>)> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let write_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        let read_file = std::fs::File::open(&path)?;

        let (title_tx, title_rx) = mpsc::sync_channel::<String>(1);
        let written = Arc::new(AtomicU64::new(0));
        let total_len = Arc::new(AtomicU64::new(0));
        let last_chunk_at = Arc::new(AtomicU64::new(0));
        let download_done = Arc::new(AtomicBool::new(false));
        let dead_url = Arc::new(AtomicBool::new(false));

        let written_task = Arc::clone(&written);
        let total_task = Arc::clone(&total_len);
        let last_chunk_task = Arc::clone(&last_chunk_at);
        let done_task = Arc::clone(&download_done);
        let dead_task = Arc::clone(&dead_url);
        let task = handle.spawn(async move {
            if let Err(e) = download_to_file(
                url,
                custom_headers,
                write_file,
                written_task,
                total_task,
                last_chunk_task,
                done_task,
                dead_task,
            )
            .await
            {
                tracing::error!("File-backed download failed: {e}");
            }
            drop(title_tx);
        });
        let download = FileBackedDownload {
            task,
            path,
            download_done: Arc::clone(&download_done),
        };

        Ok((
            Self {
                file: read_file,
                pos: 0,
                written,
                total_len,
                last_chunk_at,
                download_done,
                dead_url,
            },
            download,
            title_rx,
        ))
    }

    pub fn last_chunk_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.last_chunk_at)
    }

    pub fn download_done_arc(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.download_done)
    }

    pub fn dead_url_arc(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.dead_url)
    }

    pub fn written_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.written)
    }

    pub fn total_len_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.total_len)
    }

    pub fn wait_for_bytes(&self, target: u64, mut on_progress: impl FnMut(f32)) {
        let start = std::time::Instant::now();
        loop {
            let written = self.written.load(Ordering::Acquire);
            let total = self.total_len.load(Ordering::Acquire);
            let goal = if total > 0 { target.min(total) } else { target };
            if written >= goal
                || self.download_done.load(Ordering::Acquire)
                || self.dead_url.load(Ordering::Acquire)
            {
                on_progress(1.0);
                return;
            }
            on_progress((written as f32 / goal.max(1) as f32).min(1.0));
            if start.elapsed() > std::time::Duration::from_secs(30) {
                tracing::warn!("File-backed pre-buffer timed out, starting with partial data");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

impl Read for FileBackedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let wait_start = std::time::Instant::now();
        loop {
            let written = self.written.load(Ordering::Acquire);
            if self.pos < written {
                let max = ((written - self.pos) as usize).min(buf.len());
                self.file.seek(SeekFrom::Start(self.pos))?;
                let n = self.file.read(&mut buf[..max])?;
                self.pos += n as u64;
                return Ok(n);
            }
            if self.download_done.load(Ordering::Acquire) {
                return Ok(0);
            }
            if wait_start.elapsed() > std::time::Duration::from_secs(FILE_READ_STARVATION_SECS) {
                tracing::warn!(
                    "File-backed read starved for {FILE_READ_STARVATION_SECS}s, signaling EOF"
                );
                return Ok(0);
            }
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
    }
}

impl Seek for FileBackedReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let total = self.total_len.load(Ordering::Acquire);
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::Current(offset) => self.pos as i64 + offset,
            SeekFrom::End(offset) => {
                if total == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "stream length unknown",
                    ));
                }
                total as i64 + offset
            }
        };
        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

#[allow(clippy::too_many_arguments)]
async fn download_to_file(
    url: String,
    custom_headers: Option<HashMap<String, String>>,
    mut file: std::fs::File,
    written: Arc<AtomicU64>,
    total_len: Arc<AtomicU64>,
    last_chunk_at: Arc<AtomicU64>,
    download_done: Arc<AtomicBool>,
    dead_url: Arc<AtomicBool>,
) -> Result<(), reqwest::Error> {
    use std::io::Write;

    let client = match reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            download_done.store(true, Ordering::Release);
            return Err(e);
        }
    };

    let mut expected_total: Option<u64> = None;
    let mut current_offset: u64 = 0;
    let mut retry_count: u32 = 0;
    let mut disk_error = false;

    use futures_util::StreamExt;

    loop {
        tracing::info!("File download: connecting to {url} (offset={current_offset})");

        let req_headers = build_request_headers(custom_headers.as_ref(), current_offset);
        let resp = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get(&url).headers(req_headers).send(),
        )
        .await
        {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                tracing::warn!("File download: request failed: {e}");
                if retry_count >= MAX_DOWNLOAD_RETRIES {
                    download_done.store(true, Ordering::Release);
                    return Err(e);
                }
                retry_count += 1;
                tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
                continue;
            }
            Err(_) => {
                tracing::warn!("File download: timed out connecting (5s without response)");
                if retry_count >= MAX_DOWNLOAD_RETRIES {
                    break;
                }
                retry_count += 1;
                tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
                continue;
            }
        };

        let status = resp.status();
        if !status.is_success() {
            tracing::error!("File download: HTTP error {status} for URL: {url}");
            if retry_count >= MAX_DOWNLOAD_RETRIES {
                if status == reqwest::StatusCode::NOT_FOUND
                    || status == reqwest::StatusCode::FORBIDDEN
                {
                    dead_url.store(true, Ordering::Release);
                }
                break;
            }
            retry_count += 1;
            tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
            continue;
        }

        if expected_total.is_none() {
            expected_total = total_stream_length(&resp, current_offset);
            if let Some(total) = expected_total {
                total_len.store(total, Ordering::Release);
                tracing::info!(total, "File download: stream length known");
            }
        }

        if file.seek(SeekFrom::Start(current_offset)).is_err() {
            tracing::error!("File download: disk seek failed, aborting");
            break;
        }

        let mut stream = resp.bytes_stream();
        let mut stream_interrupted = false;

        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await {
                Ok(Some(Ok(bytes))) => {
                    if file.write_all(&bytes).is_err() {
                        disk_error = true;
                        break;
                    }
                    current_offset += bytes.len() as u64;
                    written.store(current_offset, Ordering::Release);
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    last_chunk_at.store(now_ms, Ordering::Release);
                    retry_count = 0;
                }
                Ok(Some(Err(e))) => {
                    tracing::warn!("File download: chunk error: {e}");
                    stream_interrupted = true;
                    break;
                }
                Ok(None) => {
                    break;
                }
                Err(_) => {
                    tracing::warn!("File download: chunk read timed out (network stall)");
                    stream_interrupted = true;
                    break;
                }
            }
        }

        if disk_error {
            tracing::error!("File download: disk write failed, aborting");
            break;
        }

        let incomplete = expected_total.is_some_and(|total| current_offset < total);
        if incomplete || (expected_total.is_none() && stream_interrupted) {
            if retry_count >= MAX_DOWNLOAD_RETRIES {
                tracing::error!(
                    "File download: failed after {MAX_DOWNLOAD_RETRIES} reconnect attempts at byte {current_offset}"
                );
                break;
            }
            if incomplete {
                tracing::warn!(
                    "File download: dropped prematurely at {current_offset} / {expected_total:?}. Reconnecting..."
                );
            }
            retry_count += 1;
            tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
            continue;
        }

        break;
    }

    let _ = file.flush();
    tracing::info!("File download ended at {current_offset} bytes");
    download_done.store(true, Ordering::Release);
    Ok(())
}

fn build_request_headers(
    custom_headers: Option<&HashMap<String, String>>,
    offset: u64,
) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Icy-MetaData",
        reqwest::header::HeaderValue::from_static("1"),
    );
    match custom_headers {
        Some(custom) => {
            for (key, value) in custom {
                if let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(value),
                ) {
                    headers.insert(name, value);
                }
            }
        }
        None => {
            headers.insert(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("reverbic/0.1"),
            );
        }
    }
    if offset > 0 {
        if let Ok(value) = reqwest::header::HeaderValue::from_str(&format!("bytes={offset}-")) {
            headers.insert(reqwest::header::RANGE, value);
        }
    }
    headers
}

fn total_stream_length(resp: &reqwest::Response, current_offset: u64) -> Option<u64> {
    resp.headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|cr| cr.to_str().ok())
        .and_then(|cr| cr.split('/').nth(1))
        .and_then(|total| total.parse::<u64>().ok())
        .or_else(|| {
            resp.headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|cl| cl.to_str().ok())
                .and_then(|len| len.parse::<u64>().ok())
                .map(|len| current_offset + len)
        })
}

async fn download_stream(
    url: String,
    start_byte: u64,
    custom_headers: Option<HashMap<String, String>>,
    audio_tx: mpsc::SyncSender<Bytes>,
    title_tx: mpsc::SyncSender<String>,
    download_done: Arc<AtomicBool>,
    dead_url: Arc<AtomicBool>,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .build()?;

    let mut expected_content_length: Option<u64> = None;
    let mut current_offset: u64 = start_byte;
    let mut retry_count: u32 = 0;

    use futures_util::StreamExt;

    loop {
        tracing::info!("Connecting to stream: {url} (start_byte={current_offset})");

        let req_headers = build_request_headers(custom_headers.as_ref(), current_offset);
        let resp = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get(&url).headers(req_headers).send(),
        )
        .await
        {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                tracing::warn!("Request failed: {e}");
                if retry_count >= MAX_DOWNLOAD_RETRIES {
                    return Err(e);
                }
                retry_count += 1;
                tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
                continue;
            }
            Err(_) => {
                tracing::warn!("Timed out connecting to {url} (5s without response)");
                if retry_count >= MAX_DOWNLOAD_RETRIES {
                    return Ok(());
                }
                retry_count += 1;
                tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
                continue;
            }
        };

        if current_offset == start_byte {
            log_response_diagnostics(&url, &resp);
        }

        let status = resp.status();
        if !status.is_success() {
            tracing::error!("HTTP error {status} for URL: {url}");
            if retry_count >= MAX_DOWNLOAD_RETRIES {
                if status == reqwest::StatusCode::NOT_FOUND
                    || status == reqwest::StatusCode::FORBIDDEN
                {
                    dead_url.store(true, Ordering::Release);
                }
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
            continue;
        }

        if expected_content_length.is_none() {
            expected_content_length = total_stream_length(&resp, current_offset);
        }

        let metaint = parse_icy_metaint(&resp);
        if current_offset == start_byte {
            tracing::info!("icy-metaint={metaint}, expected_length={expected_content_length:?}");
        }

        let mut stripper = IcyStripper::new(metaint, title_tx.clone());
        let mut stream = resp.bytes_stream();
        let mut first_chunk_logged = false;
        let mut stream_interrupted = false;
        let mut receiver_closed = false;

        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await {
                Ok(Some(Ok(raw_bytes))) => {
                    if !first_chunk_logged && current_offset == start_byte {
                        log_first_chunk(&raw_bytes);
                        first_chunk_logged = true;
                    }

                    current_offset += raw_bytes.len() as u64;

                    let mut audio = Vec::with_capacity(raw_bytes.len());
                    stripper.process(&raw_bytes, &mut audio);

                    if audio.is_empty() {
                        continue;
                    }

                    if audio_tx.send(Bytes::from(audio)).is_err() {
                        tracing::debug!("Receiver closed");
                        receiver_closed = true;
                        break;
                    }

                    retry_count = 0;
                }
                Ok(Some(Err(e))) => {
                    tracing::warn!("Chunk error: {e}");
                    stream_interrupted = true;
                    break;
                }
                Ok(None) => {
                    break;
                }
                Err(_) => {
                    tracing::warn!("Chunk read timed out (network stall)");
                    stream_interrupted = true;
                    break;
                }
            }
        }

        if receiver_closed {
            break;
        }

        let incomplete = expected_content_length.is_some_and(|expected| current_offset < expected);
        if incomplete || (expected_content_length.is_none() && stream_interrupted) {
            if retry_count >= MAX_DOWNLOAD_RETRIES {
                tracing::error!(
                    "Stream failed after {MAX_DOWNLOAD_RETRIES} reconnect attempts at byte {current_offset}"
                );
                break;
            }
            if incomplete {
                tracing::warn!(
                    "Stream dropped prematurely at {current_offset} / {expected_content_length:?}. Reconnecting..."
                );
            }
            retry_count += 1;
            tokio::time::sleep(DOWNLOAD_RETRY_PAUSE).await;
            continue;
        }

        break;
    }

    tracing::info!("Stream ended at {current_offset} bytes");
    download_done.store(true, Ordering::Release);
    Ok(())
}

async fn download_preview(
    url: String,
    audio_tx: mpsc::SyncSender<Bytes>,
) -> Result<(), reqwest::Error> {
    tracing::info!("Preview: starting download from {url}");

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
        .unwrap_or("unknown")
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
        tracing::warn!("Preview: HTTP {status} - aborting download");
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
                    "Preview: chunk #{chunk_count} error \
                     (raw={total_raw} audio={total_audio} expected={content_length:?}): {e}"
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
                    "Preview: ID3v2 detected - tag={skip_remaining}B \
                     ({:.1}% of estimated file)",
                    content_length
                        .map(|cl| skip_remaining as f32 / cl as f32 * 100.0)
                        .unwrap_or(0.0)
                );
            } else {
                tracing::debug!("Preview: no ID3v2 tag; audio starts at byte 0");
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
                "Preview: decoder closed the channel - \
                 raw={total_raw} audio={total_audio} chunks={chunk_count}"
            );
            return Ok(());
        }
    }

    match content_length {
        Some(expected) if total_raw < expected => {
            tracing::warn!(
                "Preview: truncated download - received={total_raw}B expected={expected}B \
                 missing={}B | audio_sent={total_audio}B chunks={chunk_count}",
                expected.saturating_sub(total_raw)
            );
        }
        Some(expected) => {
            tracing::info!(
                "Preview: download complete - raw={total_raw}B/{expected}B \
                 audio={total_audio}B chunks={chunk_count}"
            );
        }
        None => {
            tracing::info!(
                "Preview: download complete (no Content-Length) - \
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
        content_length = ?resp.headers().get(reqwest::header::CONTENT_LENGTH),
        content_range = ?resp.headers().get(reqwest::header::CONTENT_RANGE),
        icy_name   = ?resp.headers().get("icy-name"),
        icy_genre  = ?resp.headers().get("icy-genre"),
        icy_br     = ?resp.headers().get("icy-br"),
        icy_metaint = ?resp.headers().get("icy-metaint"),
        "HTTP response received"
    );
}

fn log_first_chunk(bytes: &Bytes) {
    let preview = &bytes[..bytes.len().min(16)];
    match std::str::from_utf8(&bytes[..bytes.len().min(64)]) {
        Ok(text) => tracing::warn!(
            first_bytes_utf8 = text,
            total_chunk_len = bytes.len(),
            "First chunk is text - possible HTML response"
        ),
        Err(_) => tracing::info!(
            first_bytes_hex = %preview.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "),
            total_chunk_len = bytes.len(),
            "First chunk is binary - looks like audio"
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
        let title = rx.try_recv().expect("title should have been emitted");
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
        assert!(rx.try_recv().is_err(), "no title expected with meta_len=0");
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
            .expect("title emitted after completing the block");
        assert_eq!(title, "Chunked");
    }
}
