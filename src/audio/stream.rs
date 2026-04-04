
use bytes::Bytes;
use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{mpsc, Mutex};

use crate::metadata::parse_icy_title;
pub struct StreamReader {
    rx:  Mutex<mpsc::Receiver<Bytes>>,
    buf: VecDeque<u8>,
}

impl StreamReader {
    pub fn connect(
        url: String,
        handle: tokio::runtime::Handle,
    ) -> (Self, mpsc::Receiver<String>) {
        let (audio_tx, audio_rx) = mpsc::sync_channel::<Bytes>(64);
        let (title_tx, title_rx) = mpsc::sync_channel::<String>(8);

        handle.spawn(async move {
            if let Err(e) = download_stream(url, audio_tx, title_tx).await {
                tracing::error!("Stream download failed: {e}");
            }
        });

        let reader = Self {
            rx:  Mutex::new(audio_rx),
            buf: VecDeque::new(),
        };
        (reader, title_rx)
    }
    pub fn connect_preview(
        url: String,
        handle: tokio::runtime::Handle,
    ) -> Self {
        let (audio_tx, audio_rx) = mpsc::sync_channel::<Bytes>(64);

        handle.spawn(async move {
            if let Err(e) = download_preview(url, audio_tx).await {
                tracing::error!("Preview download failed: {e}");
            }
        });

        Self {
            rx:  Mutex::new(audio_rx),
            buf: VecDeque::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Descarga HTTP con diagnóstico y stripping ICY
// ---------------------------------------------------------------------------

async fn download_stream(
    url: String,
    audio_tx: mpsc::SyncSender<Bytes>,
    title_tx: mpsc::SyncSender<String>,
) -> Result<(), reqwest::Error> {
    tracing::info!("Conectando a stream: {url}");

    let mut req_headers = reqwest::header::HeaderMap::new();
    req_headers.insert("Icy-MetaData", reqwest::header::HeaderValue::from_static("1"));
    req_headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("reverbic/0.1"),
    );

    let client = reqwest::Client::builder()
        .default_headers(req_headers)
        .build()?;

    let resp = client.get(&url).send().await?;

    log_response_diagnostics(&url, &resp);

    let status = resp.status();
    if !status.is_success() {
        tracing::error!("HTTP error {status} para URL: {url}");
        return Ok(());
    }

    let metaint = parse_icy_metaint(&resp);
    tracing::info!("icy-metaint={metaint}");

    let mut stripper = IcyStripper::new(metaint, title_tx);
    let mut stream = resp.bytes_stream();
    let mut total_audio_bytes: usize = 0;
    let mut first_chunk_logged = false;

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
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
                tracing::error!("Error leyendo chunk ({total_audio_bytes} bytes): {e}");
                break;
            }
        }
    }

    tracing::info!("Stream terminado: {total_audio_bytes} bytes de audio totales");
    Ok(())
}
async fn download_preview(
    url: String,
    audio_tx: mpsc::SyncSender<Bytes>,
) -> Result<(), reqwest::Error> {
    tracing::info!("Preview: iniciando descarga desde {url}");

    let client = reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        // connect_timeout solo para la conexión TCP; sin timeout de body (streaming)
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;

    let resp = client.get(&url).send().await?;
    let status = resp.status();

    // Loguear headers relevantes para diagnóstico de cortes
    let content_length = resp.headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());
    let content_type = resp.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("desconocido")
        .to_string();
    let transfer_encoding = resp.headers()
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

    // Bytes a saltar al inicio (ID3v2 header + body).
    let mut skip_remaining: usize = 0;
    let mut header_buf: Vec<u8> = Vec::new();
    let mut header_analyzed = false;
    let mut total_raw: usize = 0;   // bytes totales recibidos del servidor
    let mut total_audio: usize = 0; // bytes de audio enviados al decoder (sin ID3v2)
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
                    | ((header_buf[8] as usize) <<  7)
                    |  (header_buf[9] as usize);
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

    // Resumen final — crítico para detectar descargas truncadas
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
            total_chunk_len  = bytes.len(),
            "Primer chunk es texto — posible respuesta HTML"
        ),
        Err(_) => tracing::info!(
            first_bytes_hex = %preview.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "),
            total_chunk_len = bytes.len(),
            "Primer chunk es binario — parece audio"
        ),
    }
}

// ---------------------------------------------------------------------------
// IcyStripper — máquina de estados para el protocolo ICY metadata
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum IcyState {
    Audio(usize),
    MetaLen,
    MetaData(usize),
}

struct IcyStripper {
    state:    IcyState,
    metaint:  usize,
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
    fn process(&mut self, input: &[u8], output: &mut Vec<u8>) {
        for &byte in input {
            // IcyState es Copy: la copia se hace para el match; self queda libre.
            // El flag `emit` evita llamar self.emit_title() dentro del match (donde
            // la asignación a self.state todavía no completó).
            let mut emit = false;
            self.state = match self.state {
                IcyState::Audio(remaining) => {
                    output.push(byte);
                    let next = remaining - 1;
                    if next == 0 && self.metaint > 0 {
                        IcyState::MetaLen
                    } else {
                        IcyState::Audio(next)
                    }
                }

                IcyState::MetaLen => {
                    let meta_bytes = byte as usize * 16;
                    if meta_bytes == 0 {
                        IcyState::Audio(self.metaint)
                    } else {
                        self.meta_buf.clear();
                        self.meta_buf.reserve(meta_bytes);
                        IcyState::MetaData(meta_bytes)
                    }
                }

                IcyState::MetaData(remaining) => {
                    self.meta_buf.push(byte);
                    let next = remaining - 1;
                    if next == 0 {
                        emit = true;
                        IcyState::Audio(self.metaint)
                    } else {
                        IcyState::MetaData(next)
                    }
                }
            };

            if emit {
                self.emit_title();
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        // metaint = 8 — cada 8 bytes de audio aparece un bloque ICY
        let metaint = 8usize;
        let (mut s, rx) = make_stripper(metaint);

        // Construimos: 8 bytes de audio + bloque ICY con título
        let title_str = b"StreamTitle='Test Artist - Test Track';StreamUrl='';";
        // El byte de longitud ICY es ceil(len / 16); rellenamos con nulos hasta múltiplo de 16
        let padded_len = ((title_str.len() + 15) / 16) * 16;
        let mut meta_block = vec![0u8; padded_len];
        meta_block[..title_str.len()].copy_from_slice(title_str);
        let meta_len_byte = (padded_len / 16) as u8;

        let audio = b"AUDIOBYT"; // exactamente 8 bytes
        let mut input = Vec::new();
        input.extend_from_slice(audio);
        input.push(meta_len_byte);
        input.extend_from_slice(&meta_block);

        let mut out = Vec::new();
        s.process(&input, &mut out);

        // Solo los 8 bytes de audio deben pasar
        assert_eq!(out, audio);

        // El título debe haberse emitido
        let title = rx.try_recv().expect("debería haber un título");
        assert_eq!(title, "Test Artist - Test Track");
    }
    #[test]
    fn empty_metadata_block() {
        let metaint = 4usize;
        let (mut s, rx) = make_stripper(metaint);

        // 4 bytes de audio + byte de meta_len=0 + 4 bytes de audio más
        let mut input = vec![1u8, 2, 3, 4, 0u8, 5, 6, 7, 8];
        let mut out = Vec::new();
        s.process(&mut input, &mut out);

        assert_eq!(out, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(rx.try_recv().is_err(), "no title expected con meta_len=0");
    }
    #[test]
    fn state_survives_split_chunks() {
        let metaint = 4usize;
        let (mut s, rx) = make_stripper(metaint);

        let title_str = b"StreamTitle='Chunked';StreamUrl='';";
        let padded_len = ((title_str.len() + 15) / 16) * 16;
        let mut meta_block = vec![0u8; padded_len];
        meta_block[..title_str.len()].copy_from_slice(title_str);
        let meta_len_byte = (padded_len / 16) as u8;

        // Chunk 1: 2 bytes de audio
        let mut out = Vec::new();
        s.process(&[10, 20], &mut out);
        assert_eq!(out, [10, 20]);

        // Chunk 2: 2 bytes de audio + byte de longitud ICY (estado cruza chunks)
        out.clear();
        s.process(&[30, 40, meta_len_byte], &mut out);
        assert_eq!(out, [30, 40]);

        // Chunk 3: el bloque de metadata partido en dos
        out.clear();
        let mid = meta_block.len() / 2;
        s.process(&meta_block[..mid], &mut out);
        assert!(out.is_empty());

        s.process(&meta_block[mid..], &mut out);
        assert!(out.is_empty());

        let title = rx.try_recv().expect("título emitido tras completar el bloque");
        assert_eq!(title, "Chunked");
    }
}

// ---------------------------------------------------------------------------
// Read / Seek
// ---------------------------------------------------------------------------

impl Read for StreamReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let rx = self.rx.lock().expect("StreamReader mutex poisoned");
        while self.buf.len() < out.len() {
            match rx.recv() {
                Ok(bytes) => self.buf.extend(bytes.iter()),
                Err(_) => break,
            }
        }
        drop(rx);

        let n = out.len().min(self.buf.len());
        for byte in out.iter_mut().take(n) {
            *byte = self.buf.pop_front().unwrap_or(0);
        }
        Ok(n)
    }
}

impl Seek for StreamReader {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "stream is not seekable"))
    }
}
