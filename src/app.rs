
use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

use crate::audio::{AudioPlayer, PlayerCommand, PlayerState, PlayerStatus};
use crate::config::Config;
use crate::library::{self, SaveResult};
use crate::station::{all_stations, Station};

// ---------------------------------------------------------------------------
// Utilidades de zona horaria (Bélgica CET/CEST → local)
// ---------------------------------------------------------------------------
fn brussels_offset_secs() -> i32 {
    let now = Local::now();
    let (y, m, d, h) = (now.year(), now.month(), now.day(), now.hour());
    if m < 3 || m > 10 { return 3600; }
    if m > 3 && m < 10 { return 7200; }
    let last_sun = last_sunday_of_month(y, m);
    if m == 3 {
        if d > last_sun || (d == last_sun && h >= 2) { 7200 } else { 3600 }
    } else {
        if d > last_sun || (d == last_sun && h >= 3) { 3600 } else { 7200 }
    }
}
fn last_sunday_of_month(year: i32, month: u32) -> u32 {
    use chrono::Datelike;
    let (nm, ny) = if month == 12 { (1, year + 1) } else { (month + 1, year) };
    let last = NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .expect("fecha válida");
    last.day().saturating_sub(last.weekday().num_days_from_sunday())
}
fn brussels_hhmm_to_local(hhmm: &str) -> String {
    let offset = match FixedOffset::east_opt(brussels_offset_secs()) {
        Some(o) => o,
        None    => return hhmm.to_string(),
    };
    let time = match NaiveTime::parse_from_str(hhmm, "%H:%M") {
        Ok(t)  => t,
        Err(_) => return hhmm.to_string(),
    };
    let naive_dt = Local::now().date_naive().and_time(time);
    match offset.from_local_datetime(&naive_dt).single() {
        Some(dt) => dt.with_timezone(&Local).format("%H:%M").to_string(),
        None     => hhmm.to_string(),
    }
}
fn utc_to_local_hhmm(utc_str: &str) -> String {
    DateTime::parse_from_rfc3339(utc_str)
        .map(|dt| dt.with_timezone(&Local).format("%H:%M").to_string())
        .unwrap_or_else(|_| "??:??".to_string())
}

// ---------------------------------------------------------------------------
// Polling de metadata API oficial (Tomorrowland)
// ---------------------------------------------------------------------------
async fn fetch_schedule(client: &reqwest::Client, url: &str) -> Option<serde_json::Value> {
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() { return None; }
    serde_json::from_str(&resp.text().await.ok()?).ok()
}
fn current_show_time(schedule: &serde_json::Value) -> Option<String> {
    // Hora actual en Bélgica como "HH:MM" para comparar con el schedule
    let offset = FixedOffset::east_opt(brussels_offset_secs())?;
    let now_brussels = Local::now().with_timezone(&offset).format("%H:%M").to_string();

    // Día de la semana en inglés en minúsculas ("monday", "tuesday", ...)
    let day = Local::now().format("%A").to_string().to_lowercase();
    let shows = schedule[&day].as_array()?;

    // Último programa cuyo startTime <= now_brussels
    let pos = shows.iter().rposition(|e| {
        e["startTime"].as_str().map(|t| t <= now_brussels.as_str()).unwrap_or(false)
    })?;

    let start = shows[pos]["startTime"].as_str()?;
    let end   = shows.get(pos + 1).and_then(|e| e["startTime"].as_str());

    let start_local = brussels_hhmm_to_local(start);
    let end_local   = end.map(brussels_hhmm_to_local).unwrap_or_else(|| "…".to_string());

    Some(format!("{start_local} - {end_local}"))
}
fn parse_history(body: &str) -> Vec<String> {
    let arr: serde_json::Value = match serde_json::from_str(body) {
        Ok(v)  => v,
        Err(_) => return Vec::new(),
    };
    arr.as_array()
        .map(|entries| {
            entries.iter().take(10).filter_map(|e| {
                let artist = e["artist"].as_str().unwrap_or("");
                let title  = e["title"].as_str()?;
                let ts     = e["timestamp"].as_str().unwrap_or("");
                let time   = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| {
                        Utc.from_utc_datetime(&ndt)
                            .with_timezone(&Local)
                            .format("%H:%M")
                            .to_string()
                    })
                    .unwrap_or_else(|_| "??:??".to_string());
                Some(if artist.is_empty() {
                    format!("{time}  {title}")
                } else {
                    format!("{time}  {artist} - {title}")
                })
            }).collect()
        })
        .unwrap_or_default()
}
fn parse_api_response(
    body: &str,
    history_body: Option<&str>,
    schedule: Option<&serde_json::Value>,
) -> Option<PlayerCommand> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let title  = v["title"].as_str()?.to_string();
    let artist = v["artist"].as_str().unwrap_or("").to_string();

    // OnlyHit: show es un objeto {name, permalink, ...}; Tomorrowland: string directo
    let show_name = if v["show"].is_object() {
        v["show"]["name"].as_str().unwrap_or("").to_string()
    } else {
        v["show"].as_str().unwrap_or("").to_string()
    };

    // Enriquecer el show con el horario local si está disponible (solo Tomorrowland)
    let show = match schedule.and_then(current_show_time) {
        Some(time) => format!("{show_name}  {time}"),
        None       => show_name,
    };

    let recent = if let Some(hist) = history_body {
        // OnlyHit: historial viene de endpoint separado
        parse_history(hist)
    } else {
        // Tomorrowland: historial embebido en `tracklog`
        v["tracklog"]
            .as_array()
            .map(|entries| {
                entries.iter().take(10).filter_map(|e| {
                    let t     = e["title"].as_str()?;
                    let a     = e["artist"].as_str().unwrap_or("");
                    let start = e["startTime"].as_str().unwrap_or("");
                    let time  = utc_to_local_hhmm(start);
                    Some(if a.is_empty() {
                        format!("{time}  {t}")
                    } else {
                        format!("{time}  {a} - {t}")
                    })
                }).collect()
            })
            .unwrap_or_default()
    };

    Some(PlayerCommand::ApiMetadata { title, artist, show, recent })
}
async fn poll_metadata_loop(
    url: String,
    history_url: Option<String>,
    schedule_url: Option<String>,
    cmd_tx: mpsc::Sender<PlayerCommand>,
) {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c)  => c,
        Err(e) => {
            tracing::error!("No se pudo crear cliente HTTP para metadata: {e}");
            return;
        }
    };

    // Schedule se descarga una sola vez al inicio; se reutiliza en todos los polls
    let schedule = if let Some(ref s_url) = schedule_url {
        let s = fetch_schedule(&client, s_url).await;
        if s.is_none() {
            tracing::warn!("No se pudo obtener el schedule; se mostrará solo el nombre del show");
        }
        s
    } else {
        None
    };

    loop {
        // Fetch historial separado si la estación lo tiene (e.g. OnlyHit)
        let history_body: Option<String> = if let Some(ref h_url) = history_url {
            match client.get(h_url).send().await {
                Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
                Ok(resp) => {
                    tracing::warn!("History API HTTP {}", resp.status());
                    None
                }
                Err(e) => {
                    tracing::warn!("History API no disponible: {e}");
                    None
                }
            }
        } else {
            None
        };

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(body) => {
                        if let Some(cmd) = parse_api_response(&body, history_body.as_deref(), schedule.as_ref()) {
                            if cmd_tx.send(cmd).await.is_err() {
                                break; // canal cerrado — audio thread muerto
                            }
                        }
                    }
                    Err(e) => tracing::warn!("Error leyendo body de metadata API: {e}"),
                }
            }
            Ok(resp) => tracing::warn!("Metadata API HTTP {}: usando ICY como fallback", resp.status()),
            Err(e)   => tracing::warn!("Metadata API no disponible ({e}): usando ICY como fallback"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

// ---------------------------------------------------------------------------
// Integración Deezer — búsqueda de preview de 30s
// ---------------------------------------------------------------------------
fn strip_version_info(title: &str) -> String {
    const VERSION_KEYWORDS: &[&str] = &[
        "remix", "edit", "mix", "version", "remaster", "live", "extended",
        "radio", "original", "club", "vip", "instrumental", "acoustic",
        "bootleg", "rework", "flip", "dub", "remi",
    ];
    let mut result = title.to_string();
    // Itera hasta que no haya más cambios (puede haber múltiples paréntesis)
    loop {
        let before = result.clone();
        for (open, close) in [('(', ')'), ('[', ']')] {
            if let Some(start) = result.find(open) {
                if let Some(rel_end) = result[start..].find(close) {
                    let end = start + rel_end;
                    let inner = result[start + 1..end].to_lowercase();
                    if VERSION_KEYWORDS.iter().any(|kw| inner.contains(kw)) {
                        let prefix = result[..start].trim_end();
                        let suffix = result[end + 1..].trim_start();
                        result = if suffix.is_empty() {
                            prefix.to_string()
                        } else {
                            format!("{prefix} {suffix}")
                        };
                    }
                }
            }
        }
        if result == before { break; }
    }
    result.trim().to_string()
}
fn log_deezer_not_found(raw: &str, query: &str) {
    use std::io::Write;
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let path = std::path::PathBuf::from(home)
        .join(".reverbic")
        .join("deezer_not_found.log");

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let ts = Local::now().format("%Y-%m-%dT%H:%M:%S");
    let line = format!("{ts}  original: \"{raw}\"  query: {query}\n");

    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}
async fn deezer_preview(raw: &str) -> Option<(String, String)> {
    // Quitar prefijo de timestamp "HH:MM  " si existe
    let clean = raw.splitn(2, "  ").nth(1).unwrap_or(raw).trim();

    // Construir query Deezer con campos precisos
    let q = if let Some(sep) = clean.find(" - ") {
        let raw_artist = clean[..sep].trim();
        let raw_title  = clean[sep + 3..].trim();

        // Tomar solo el artista principal (antes de la primera "," o "&")
        let primary_artist = raw_artist
            .split([',', '&'])
            .next()
            .unwrap_or(raw_artist)
            .trim();

        // Quitar info de remix/versión del título
        let clean_title = strip_version_info(raw_title);

        tracing::debug!("Deezer query: artist='{primary_artist}' track='{clean_title}' (original: '{clean}')");
        format!(r#"artist:"{primary_artist}" track:"{clean_title}""#)
    } else {
        strip_version_info(clean)
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("reverbic/0.1")
        .build()
        .ok()?;

    let search_resp = client
        .get("https://api.deezer.com/search")
        .query(&[("q", q.as_str()), ("limit", "1")])
        .send()
        .await
        .ok()?;

    if !search_resp.status().is_success() {
        tracing::warn!("Deezer search HTTP {}", search_resp.status());
        return None;
    }

    let body = search_resp.text().await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    let data = json["data"].as_array()?;
    if data.is_empty() {
        tracing::warn!("Deezer: sin resultado para query '{q}'");
        log_deezer_not_found(raw, &q);
        return None;
    }
    let track = data.first()?;
    let preview_url = track["preview"].as_str()?;
    if preview_url.is_empty() {
        tracing::warn!("Deezer: track encontrado pero sin preview URL");
        return None;
    }

    let artist        = track["artist"]["name"].as_str().unwrap_or("");
    let title         = track["title"].as_str().unwrap_or("");
    let display_title = if artist.is_empty() { title.to_string() } else { format!("{artist} - {title}") };

    tracing::info!("Deezer: preview encontrado para '{}' — {preview_url}", display_title);
    Some((preview_url.to_string(), display_title))
}

pub enum AppScreen {
    StationList,
    Playing,
}
pub enum AppFocus {
    Stations,
    RecentTracks,
}

pub struct App {
    pub screen:          AppScreen,
    pub stations:        &'static [Station],
    pub selected:        usize,
    pub player:          AudioPlayer,
    pub should_quit:     bool,
    pub focus:           AppFocus,
    pub recent_selected: usize,
    pub saved_tracks:    Vec<String>,
    pub save_notice:     Option<String>,
    config:              Config,
    metadata_task:       Option<tokio::task::JoinHandle<()>>,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load();
        let player = AudioPlayer::spawn();

        // Aplicar volumen guardado al arrancar
        player.send(PlayerCommand::SetVolume(config.volume)).await;

        let last = config.last_selected.min(all_stations().len().saturating_sub(1));

        Self {
            screen:          AppScreen::StationList,
            stations:        all_stations(),
            selected:        last,
            player,
            should_quit:     false,
            focus:           AppFocus::Stations,
            recent_selected: 0,
            saved_tracks:    Vec::new(),
            save_notice:     None,
            config,
            metadata_task:   None,
        }
    }

    fn start_metadata_polling(
        &mut self,
        url: &'static str,
        history_url: Option<&'static str>,
        schedule_url: Option<&'static str>,
    ) {
        self.stop_metadata_polling();
        let cmd_tx = self.player.clone_sender();
        self.metadata_task = Some(tokio::spawn(poll_metadata_loop(
            url.to_string(),
            history_url.map(str::to_string),
            schedule_url.map(str::to_string),
            cmd_tx,
        )));
    }

    fn stop_metadata_polling(&mut self) {
        if let Some(handle) = self.metadata_task.take() {
            handle.abort();
        }
    }
    pub async fn on_key(&mut self, key: KeyCode) {
        // Limpiar el notice en cualquier tecla nueva
        self.save_notice = None;

        // Teclas globales — actúan sin importar el foco
        match key {
            KeyCode::Char(' ') => {
                match self.player.state().status {
                    PlayerStatus::Playing => { self.player.send(PlayerCommand::Pause).await; }
                    PlayerStatus::Paused  => { self.player.send(PlayerCommand::Resume).await; }
                    _ => {}
                }
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let new_vol = (self.player.state().volume + 0.05).min(1.0);
                self.player.send(PlayerCommand::SetVolume(new_vol)).await;
                return;
            }
            KeyCode::Char('-') => {
                let new_vol = (self.player.state().volume - 0.05).max(0.0);
                self.player.send(PlayerCommand::SetVolume(new_vol)).await;
                return;
            }
            KeyCode::Char('q') => {
                let state = self.player.state();
                self.config.volume = state.volume;
                self.config.last_selected = self.selected;
                self.config.save();
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.should_quit = true;
                return;
            }
            KeyCode::Tab => {
                let has_recent = !self.player.state().recent_titles.is_empty();
                if has_recent {
                    self.focus = match self.focus {
                        AppFocus::Stations    => AppFocus::RecentTracks,
                        AppFocus::RecentTracks => AppFocus::Stations,
                    };
                    self.recent_selected = 0;
                }
                return;
            }
            _ => {}
        }

        match self.focus {
            AppFocus::Stations => self.on_key_stations(key).await,
            AppFocus::RecentTracks => self.on_key_recent(key).await,
        }
    }

    async fn on_key_stations(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.stations.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                let station = self.stations[self.selected].clone();
                self.stop_metadata_polling();
                if self.player.send(PlayerCommand::Play(station.clone())).await {
                    self.screen = AppScreen::Playing;
                    self.saved_tracks = library::load_saved_tracks(station.key);
                    if let Some(api_url) = station.metadata_api_url {
                        self.start_metadata_polling(api_url, station.history_api_url, station.schedule_url);
                    }
                }
            }
            KeyCode::Char('s') => {
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.saved_tracks = Vec::new();
                self.screen = AppScreen::StationList;
            }
            KeyCode::Esc => {
                let state = self.player.state();
                self.config.volume = state.volume;
                self.config.last_selected = self.selected;
                self.config.save();
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.should_quit = true;
            }
            _ => {}
        }
    }

    async fn on_key_recent(&mut self, key: KeyCode) {
        let titles = self.player.state().recent_titles;
        let len = titles.len();
        if len == 0 {
            self.focus = AppFocus::Stations;
            return;
        }
        // Clamp por si la lista encogió desde la última vez
        self.recent_selected = self.recent_selected.min(len - 1);

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.recent_selected = self.recent_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.recent_selected + 1 < len {
                    self.recent_selected += 1;
                }
            }
            KeyCode::Enter => {
                let title = titles[self.recent_selected].clone();
                let key = self.player.state().station.as_ref().map(|s| s.key).unwrap_or("unknown");
                match library::save_track(&title, key) {
                    SaveResult::Saved => {
                        self.saved_tracks = library::load_saved_tracks(key);
                        self.save_notice = Some(format!("Guardado: {title}"));
                    }
                    SaveResult::AlreadySaved => {
                        self.save_notice = Some(format!("Ya guardada: {title}"));
                    }
                }
            }
            KeyCode::Char('p') => {
                let state = self.player.state();
                if state.preview_title.is_some() || state.preview_searching {
                    // Preview activo o buscando → parar / cancelar
                    self.player.send(PlayerCommand::StopPreview).await;
                    self.player.send(PlayerCommand::SetPreviewSearching(false)).await;
                } else if !titles.is_empty() {
                    let raw = titles[self.recent_selected].clone();
                    let cmd_tx = self.player.clone_sender();
                    // Feedback inmediato: spinner en la fila + texto "Buscando..." en help bar
                    let _ = cmd_tx.send(PlayerCommand::SetPreviewSearching(true)).await;
                    let _ = cmd_tx.send(PlayerCommand::SetPreviewLoadingTrack(Some(raw.clone()))).await;
                    tokio::spawn(async move {
                        match deezer_preview(&raw).await {
                            Some((url, title)) => {
                                // Quitar spinner: ya va a reproducir
                                let _ = cmd_tx.send(PlayerCommand::SetPreviewLoadingTrack(None)).await;
                                // Streaming: el audio empieza a reproducirse de inmediato
                                // sin esperar a que se descargue el archivo completo.
                                if cmd_tx.send(PlayerCommand::PlayPreview { url, title, raw_track: raw }).await.is_err() {
                                    return;
                                }
                                // Auto-stop de seguridad: los previews de Deezer duran ~30s.
                                // El audio termina solo cuando el stream llega a EOF,
                                // pero limpiamos el estado de UI tras 35s como fallback.
                                tokio::time::sleep(std::time::Duration::from_secs(35)).await;
                                let _ = cmd_tx.send(PlayerCommand::StopPreview).await;
                            }
                            None => {
                                tracing::warn!("Deezer: sin resultado para '{raw}'");
                                // Quitar spinner y marcar como no disponible para esta sesión
                                let _ = cmd_tx.send(PlayerCommand::SetPreviewLoadingTrack(None)).await;
                                let _ = cmd_tx.send(PlayerCommand::SetPreviewSearching(false)).await;
                                let _ = cmd_tx.send(PlayerCommand::MarkPreviewUnavailable(raw)).await;
                            }
                        }
                    });
                }
            }
            KeyCode::Esc => {
                self.focus = AppFocus::Stations;
            }
            _ => {}
        }
    }

    pub fn player_state(&self) -> PlayerState {
        self.player.state()
    }
}
