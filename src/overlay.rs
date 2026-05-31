#![cfg(target_os = "windows")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, watch};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        Media::Audio::{
            AudioSessionStateActive, AudioSessionStateExpired,
            IAudioSessionControl2, IAudioSessionEnumerator, IAudioSessionManager2,
            IMMDeviceEnumerator, MMDeviceEnumerator, eConsole, eRender,
            Endpoints::IAudioMeterInformation,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
            Console::GetConsoleWindow,
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
                PROCESSENTRY32W, TH32CS_SNAPPROCESS,
            },
            LibraryLoader::GetModuleHandleW,
            Threading::AttachThreadInput,
        },
        UI::{
            Shell::{Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIIF_NOSOUND},
            WindowsAndMessaging::*,
        },
    },
};

use crate::audio::{PlayerCommand, PlayerState, PlayerStatus};
use crate::config::{Config, OverlayMode};

const OW: i32 = 320;
const OH: i32 = 105;
const ACCENT_W: i32 = 3;
const PAD_L:    i32 = ACCENT_W + 10;
const PAD_R:    i32 = 8;

const PEAK_HOLD_MS:     u128 = 1500;
const PEAK_DECAY_TICK:  f32  = 0.020; // por tick de 50ms → full-scale en ~2.5s
const DUCK_THRESHOLD:   f32  = 0.02;  // 2% de pico = audio activo en otro proceso
const UNDUCK_DELAY_MS:  u128 = 2000;  // silencio sostenido antes de restaurar

// ── Palette (COLORREF: little-endian BGR = (B<<16)|(G<<8)|R) ─────────────────
const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(((b as u32) << 16) | ((g as u32) << 8) | (r as u32))
}

const C_BG:        COLORREF = rgb(0x14, 0x14, 0x14);
const C_ACCENT:    COLORREF = rgb(0x44, 0xCC, 0x33);
const C_BRAND:     COLORREF = rgb(0xE8, 0xE8, 0xE8); // near-white, visible
const C_SEPARATOR: COLORREF = rgb(0x2A, 0x2A, 0x2A);
const C_STATION:   COLORREF = rgb(0x44, 0xCC, 0x44);
const C_SHOW:      COLORREF = rgb(0xCC, 0xA0, 0x30);
const C_TITLE:     COLORREF = rgb(0xAA, 0xAA, 0xAA);
const C_ST_OK:     COLORREF = rgb(0x44, 0xCC, 0x33);
const C_ST_BUF:    COLORREF = rgb(0xCC, 0x99, 0x00);
const C_ST_RECO:   COLORREF = rgb(0xFF, 0x80, 0x00);
const C_VU_BG:     COLORREF = rgb(0x28, 0x28, 0x28);
const C_VU_LOW:    COLORREF = rgb(0x33, 0xAA, 0x33);
const C_VU_MED:    COLORREF = rgb(0xAA, 0xCC, 0x00);
const C_VU_HIGH:   COLORREF = rgb(0xFF, 0x44, 0x00);

// ── Internal types ────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum OStatus {
    Playing,
    Buffering(f32),
    Reconnecting(u32),
}

struct State {
    station:    String,
    show:       String,
    title:      String,
    level_db:   f32,
    ostatus:    OStatus,
    peak_ratio: f32,
    game:       String,
}

// ── Public API ────────────────────────────────────────────────────────────────

static MEDIA_VK_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<u32>> = std::sync::OnceLock::new();
static GAME_STATE:  std::sync::OnceLock<Arc<Mutex<String>>>               = std::sync::OnceLock::new();

pub fn spawn(
    state_rx:  watch::Receiver<PlayerState>,
    config_rx: watch::Receiver<Config>,
    cmd_tx:    mpsc::Sender<PlayerCommand>,
) {
    std::thread::Builder::new()
        .name("overlay".into())
        .spawn(move || unsafe {
            if let Err(e) = run(state_rx, config_rx, cmd_tx) {
                tracing::error!("overlay: {e}");
            }
        })
        .expect("overlay thread");
}

// ── Win32 thread ──────────────────────────────────────────────────────────────

unsafe fn run(
    mut rx:        watch::Receiver<PlayerState>,
    mut config_rx: watch::Receiver<Config>,
    cmd_tx:        mpsc::Sender<PlayerCommand>,
) -> windows::core::Result<()> {
    // WASAPI en hilo propio. Este hilo Win32 NO inicializa COM.
    let own_pid = std::process::id();
    {
        let shared_game = Arc::new(Mutex::new(String::new()));
        let sg = Arc::clone(&shared_game);
        // Guardamos el Arc en un OnceLock para que el paint lo lea
        GAME_STATE.get_or_init(|| shared_game);

        std::thread::Builder::new()
            .name("wasapi-monitor".into())
            .spawn(move || unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                loop {
                    let proc_map = build_process_snapshot();
                    let (_peak, detected) = detect_audio_activity(own_pid, &proc_map);
                    let raw = detected.unwrap_or_default();
                    crate::game_detect::set(if raw.is_empty() { None } else { Some(raw) });
                    let display = crate::game_detect::get_name().unwrap_or_default();
                    if let Ok(mut g) = sg.lock() { *g = display; }
                    std::thread::sleep(Duration::from_millis(500));
                }
            })
            .expect("wasapi-monitor thread");
    }

    let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);
    let class     = w!("ReverbicOverlay");

    let shared = Arc::new(Mutex::new(State {
        station:    String::new(),
        show:       String::new(),
        title:      String::new(),
        level_db:   -60.0,
        ostatus:    OStatus::Playing,
        peak_ratio: 0.0,
        game:       String::new(),
    }));
    let raw = Arc::into_raw(Arc::clone(&shared));

    let wc = WNDCLASSEXW {
        cbSize:        std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc:   Some(wnd_proc),
        hInstance:     hinstance,
        lpszClassName: class,
        ..Default::default()
    };
    RegisterClassExW(&wc);

    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
        class,
        w!("Reverbic"),
        WS_POPUP,
        16, 16, OW, OH,
        None, None, hinstance,
        Some(raw as *const _),
    )?;

    let init_alpha: u8 = (config_rx.borrow().overlay_alpha.min(100) as u32 * 255 / 100) as u8;
    SetLayeredWindowAttributes(hwnd, COLORREF(0), init_alpha, LWA_ALPHA)?;

    let mut msg           = MSG::default();
    let mut visible       = false;
    let mut peak_ratio    = 0_f32;
    let mut peak_held_at  = Instant::now();
    let mut cfg           = config_rx.borrow().clone();
    let mut playing       = false;
    let mut last_title    = String::new();

    let (media_tx, media_rx) = std::sync::mpsc::sync_channel::<u32>(4);
    let _ = MEDIA_VK_TX.set(media_tx);

    let mut hook:           HHOOK         = HHOOK::default();
    let mut tray_added:     bool          = false;
    let tray_id: u32 = 1001;
    let wm_tray      = WM_APP + 1;

    let mut is_ducked:      bool          = false;
    let mut pre_duck_vol:   f32           = 1.0;
    let mut quiet_since:    Option<Instant> = None;
    let mut next_duck_check: Instant      = Instant::now();

    // Aplicar el estado inicial de la config — has_changed() no dispara al arrancar
    if cfg.media_keys {
        hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0)
            .unwrap_or_default();
    }
    if cfg.tray_icon {
        tray_added = add_tray_icon(hwnd, tray_id, wm_tray).is_ok();
    }

    loop {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                if !hook.0.is_null() { let _ = UnhookWindowsHookEx(hook); }
                if tray_added { remove_tray_icon(hwnd, tray_id); }
                return Ok(());
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // ── Media key events from hook ────────────────────────────
        while let Ok(vk) = media_rx.try_recv() {
            let ps = rx.borrow().clone();
            let cmd = match vk {
                0xB3 => match ps.status {
                    PlayerStatus::Playing => Some(PlayerCommand::Pause),
                    PlayerStatus::Paused  => Some(PlayerCommand::Resume),
                    _                     => None,
                },
                0xB2 => Some(PlayerCommand::Stop),
                _ => None,
            };
            if let Some(c) = cmd {
                let _ = cmd_tx.blocking_send(c);
            }
        }

        // ── Config update ─────────────────────────────────────────
        if config_rx.has_changed().unwrap_or(false) {
            cfg = config_rx.borrow_and_update().clone();

            // Media keys hook
            if cfg.media_keys && hook.0.is_null() {
                hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0)
                    .unwrap_or_default();
            } else if !cfg.media_keys && !hook.0.is_null() {
                let _ = UnhookWindowsHookEx(hook);
                hook = HHOOK::default();
            }

            // Tray icon
            if cfg.tray_icon && !tray_added {
                tray_added = add_tray_icon(hwnd, tray_id, wm_tray).is_ok();
            } else if !cfg.tray_icon && tray_added {
                remove_tray_icon(hwnd, tray_id);
                tray_added = false;
            }

            // Transparencia overlay
            let new_alpha: u8 = (cfg.overlay_alpha.min(100) as u32 * 255 / 100) as u8;
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), new_alpha, LWA_ALPHA);
        }

        let mut need_repaint = false;

        // ── Player state ──────────────────────────────────────────
        if rx.has_changed().unwrap_or(false) {
            let ps = rx.borrow_and_update().clone();
            playing = matches!(
                ps.status,
                PlayerStatus::Playing | PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_)
            );

            let new_title = ps.title.clone().unwrap_or_default();
            let title_changed = new_title != last_title && !new_title.is_empty() && playing;

            if let Ok(mut s) = shared.lock() {
                s.station  = ps.station.as_ref().map(|st| st.name.clone()).unwrap_or_default();
                s.show     = ps.api_show.clone().unwrap_or_default();
                s.title    = new_title.clone();
                s.level_db = ps.level_db;
                s.ostatus  = match ps.status {
                    PlayerStatus::Buffering(f)    => OStatus::Buffering(f),
                    PlayerStatus::Reconnecting(n) => OStatus::Reconnecting(n),
                    _                             => OStatus::Playing,
                };
                let cur = ((ps.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
                if cur >= peak_ratio {
                    peak_ratio   = cur;
                    peak_held_at = Instant::now();
                }
                s.peak_ratio = peak_ratio;
            }

            // Notification on track change
            if title_changed && cfg.notifications && tray_added {
                last_title = new_title.clone();
                let station = ps.station.as_ref().map(|s| s.name.as_str()).unwrap_or("Reverbic");
                let _ = show_balloon(hwnd, tray_id, station, &new_title);
            }

            // Update tray tooltip with current track
            if tray_added {
                let tip = if !new_title.is_empty() {
                    format!("Reverbic — {new_title}")
                } else {
                    "Reverbic".to_string()
                };
                let _ = update_tray_tip(hwnd, tray_id, &tip);
            }
        }

        // ── Visibility ────────────────────────────────────────────
        let should_show = match cfg.overlay_mode {
            OverlayMode::Hidden      => false,
            OverlayMode::Always      => playing,
            OverlayMode::WhenPlaying => playing,
            OverlayMode::Games       => playing && is_fullscreen_foreground(hwnd),
        };

        if should_show != visible {
            visible = should_show;
            let _ = ShowWindow(hwnd, if should_show { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
        if visible {
            need_repaint = true;
        }

        // ── Juego activo (lee del wasapi-monitor, sin bloquear) ───────
        if let Some(gs) = GAME_STATE.get() {
            if let Ok(name) = gs.try_lock() {
                if let Ok(mut s) = shared.try_lock() {
                    if s.game != *name {
                        s.game = name.clone();
                        need_repaint = true;
                    }
                }
            }
        }

        // ── Auto-duck ─────────────────────────────────────────────
        if cfg.duck_enabled && playing {
            if Instant::now() >= next_duck_check {
                next_duck_check = Instant::now() + Duration::from_millis(500);
                let peak        = other_audio_peak(own_pid);
                let duck_target = cfg.duck_volume as f32 / 100.0;

                if peak > DUCK_THRESHOLD {
                    quiet_since = None;
                    if !is_ducked {
                        let current_vol = rx.borrow().volume;
                        if current_vol > duck_target + 0.01 {
                            pre_duck_vol = current_vol;
                            let _ = cmd_tx.blocking_send(PlayerCommand::SetVolume(duck_target));
                            is_ducked = true;
                        }
                    }
                } else if is_ducked {
                    match quiet_since {
                        None => quiet_since = Some(Instant::now()),
                        Some(t) if t.elapsed().as_millis() >= UNDUCK_DELAY_MS => {
                            let _ = cmd_tx.blocking_send(PlayerCommand::SetVolume(pre_duck_vol));
                            is_ducked    = false;
                            quiet_since  = None;
                        }
                        _ => {}
                    }
                }
            }
        } else if is_ducked {
            // duck desactivado o Reverbic detenido — restaurar inmediatamente
            let _ = cmd_tx.blocking_send(PlayerCommand::SetVolume(pre_duck_vol));
            is_ducked   = false;
            quiet_since = None;
        }

        // ── Peak decay (corre siempre, independiente de rx) ───────
        if visible && peak_held_at.elapsed().as_millis() > PEAK_HOLD_MS {
            let new_peak = (peak_ratio - PEAK_DECAY_TICK).max(0.0);
            if new_peak < peak_ratio {
                peak_ratio = new_peak;
                if let Ok(mut s) = shared.lock() {
                    s.peak_ratio = peak_ratio;
                }
                need_repaint = true;
            }
        }

        if need_repaint {
            let _ = InvalidateRect(hwnd, None, TRUE);
            let _ = UpdateWindow(hwnd);
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let cs = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<State>;
            let mut ps = PAINTSTRUCT::default();
            let hdc    = BeginPaint(hwnd, &mut ps);
            if !ptr.is_null() {
                if let Ok(s) = (*ptr).lock() {
                    paint(hdc, &s);
                }
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<State>;
            if !ptr.is_null() {
                drop(Arc::from_raw(ptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        // Mensaje de callback del tray icon
        x if x == WM_APP + 1 => {
            let event = (lparam.0 as u16) as u32;
            if event == WM_LBUTTONDBLCLK {
                let console = GetConsoleWindow();
                if !console.0.is_null() {
                    // AttachThreadInput: adjuntamos nuestro thread al foreground thread
                    // para que Windows permita el cambio de foco
                    let fg     = GetForegroundWindow();
                    let fg_tid = GetWindowThreadProcessId(fg, None);
                    let my_tid = GetWindowThreadProcessId(hwnd, None);
                    if fg_tid != 0 && fg_tid != my_tid {
                        let _ = AttachThreadInput(fg_tid, my_tid, TRUE);
                    }
                    if IsIconic(console).as_bool() {
                        let _ = ShowWindow(console, SW_RESTORE);
                    }
                    let _ = BringWindowToTop(console);
                    let _ = SetForegroundWindow(console);
                    let _ = ShowWindow(console, SW_SHOW);
                    if fg_tid != 0 && fg_tid != my_tid {
                        let _ = AttachThreadInput(fg_tid, my_tid, FALSE);
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ── Painting ──────────────────────────────────────────────────────────────────

unsafe fn paint(hdc: HDC, s: &State) {
    // Background + accent bar
    fill(hdc, RECT { left: 0, top: 0, right: OW,       bottom: OH }, C_BG);
    fill(hdc, RECT { left: 0, top: 0, right: ACCENT_W, bottom: OH }, C_ACCENT);

    SetBkMode(hdc, TRANSPARENT);

    // ── Brand "REVERBIC" (bold, near-white) + status label (right) ─
    let f_brand = font(11, true);
    let prev    = SelectObject(hdc, HGDIOBJ(f_brand.0));

    SetTextColor(hdc, C_BRAND);
    let _ = TextOutW(hdc, PAD_L, 5, &wide("REVERBIC"));

    let (status_str, status_color) = status_label(s.ostatus);
    SetTextColor(hdc, status_color);
    let prev_align = SetTextAlign(hdc, TA_RIGHT);
    let _ = TextOutW(hdc, OW - PAD_R, 5, &wide(&status_str));
    let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(prev_align));

    // ── Separador ─────────────────────────────────────────────────
    fill(hdc, RECT { left: PAD_L, top: 20, right: OW - PAD_R, bottom: 21 }, C_SEPARATOR);

    // ── Station ───────────────────────────────────────────────────
    let f_station = font(15, true);
    SelectObject(hdc, HGDIOBJ(f_station.0));
    SetTextColor(hdc, C_STATION);
    let _ = TextOutW(hdc, PAD_L, 25, &wide_truncated(&s.station, 30));

    // ── Show / Track ──────────────────────────────────────────────
    let f_detail = font(12, false);
    SelectObject(hdc, HGDIOBJ(f_detail.0));

    let has_show = !s.show.is_empty();
    if has_show {
        SetTextColor(hdc, C_SHOW);
        let _ = TextOutW(hdc, PAD_L, 45, &wide_truncated(&s.show, 36));
    }

    let y_title = if has_show { 61 } else { 45 };
    SetTextColor(hdc, C_TITLE);
    let title = if s.title.is_empty() { "—" } else { s.title.as_str() };
    let _ = TextOutW(hdc, PAD_L, y_title, &wide_truncated(title, 38));

    // ── Jugando ───────────────────────────────────────────────────
    let vu_top = if !s.game.is_empty() {
        fill(hdc, RECT { left: PAD_L, top: 77, right: OW - PAD_R, bottom: 78 }, C_SEPARATOR);
        let f_game = font(11, false);
        SelectObject(hdc, HGDIOBJ(f_game.0));
        SetTextColor(hdc, C_TITLE);
        let _ = TextOutW(hdc, PAD_L, 80, &wide("Jugando: "));
        SetTextColor(hdc, C_ACCENT);
        let _ = TextOutW(hdc, PAD_L + 52, 80, &wide_truncated(&s.game, 28));
        let _ = DeleteObject(HGDIOBJ(f_game.0));
        94
    } else { 79 };

    // ── VU meter + peak hold ──────────────────────────────────────
    let vu_x1 = PAD_L;
    let vu_x2 = OW - PAD_R;
    fill(hdc, RECT { left: vu_x1, top: vu_top, right: vu_x2, bottom: vu_top + 4 }, C_VU_BG);

    let ratio = ((s.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
    let bar_w = ((vu_x2 - vu_x1) as f32 * ratio) as i32;
    if bar_w > 0 {
        fill(hdc, RECT { left: vu_x1, top: vu_top, right: vu_x1 + bar_w, bottom: vu_top + 4 }, vu_color(ratio));
    }

    // Peak hold: marca vertical de 2px del color del nivel pico
    if s.peak_ratio > 0.02 {
        let px = (vu_x1 + ((vu_x2 - vu_x1) as f32 * s.peak_ratio) as i32).min(vu_x2 - 2);
        fill(hdc, RECT { left: px, top: vu_top, right: px + 2, bottom: vu_top + 4 }, vu_color(s.peak_ratio));
    }

    // ── Cleanup ───────────────────────────────────────────────────
    SelectObject(hdc, prev);
    let _ = DeleteObject(HGDIOBJ(f_brand.0));
    let _ = DeleteObject(HGDIOBJ(f_station.0));
    let _ = DeleteObject(HGDIOBJ(f_detail.0));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn status_label(os: OStatus) -> (String, COLORREF) {
    match os {
        OStatus::Playing         => ("● PLAYING".into(),              C_ST_OK),
        OStatus::Buffering(f)    => (format!("◌ {:.0}%", f * 100.0), C_ST_BUF),
        OStatus::Reconnecting(n) => (format!("↻ ×{n}"),              C_ST_RECO),
    }
}

fn vu_color(ratio: f32) -> COLORREF {
    if ratio < 0.5 { C_VU_LOW } else if ratio < 0.8 { C_VU_MED } else { C_VU_HIGH }
}

unsafe fn fill(hdc: HDC, r: RECT, color: COLORREF) {
    let b = CreateSolidBrush(color);
    FillRect(hdc, &r, b);
    let _ = DeleteObject(HGDIOBJ(b.0));
}

unsafe fn font(size: i32, bold: bool) -> HFONT {
    CreateFontW(
        size, 0, 0, 0,
        if bold { 700 } else { 400 },
        0, 0, 0,
        1, 0, 0,
        5, // CLEARTYPE_QUALITY
        0,
        w!("Segoe UI"),
    )
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 == WM_KEYDOWN as usize {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        if kb.vkCode == 0xB3 || kb.vkCode == 0xB2 {
            if let Some(tx) = MEDIA_VK_TX.get() {
                let _ = tx.try_send(kb.vkCode);
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

unsafe fn add_tray_icon(hwnd: HWND, id: u32, callback_msg: u32) -> windows::core::Result<()> {
    let hmodule  = GetModuleHandleW(None)?;
    let icon = LoadIconW(hmodule, PCWSTR(1 as *const u16))
        .unwrap_or_else(|_| LoadIconW(HINSTANCE::default(), IDI_APPLICATION)
            .expect("IDI_APPLICATION siempre disponible"));
    let mut tip = [0u16; 128];
    let text: Vec<u16> = "Reverbic".encode_utf16().collect();
    tip[..text.len().min(127)].copy_from_slice(&text[..text.len().min(127)]);

    let mut nid = NOTIFYICONDATAW {
        cbSize:           std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd:             hwnd,
        uID:              id,
        uFlags:           NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: callback_msg,
        hIcon:            icon,
        szTip:            tip,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_ADD, &mut nid).ok()
}

unsafe fn remove_tray_icon(hwnd: HWND, id: u32) {
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd:   hwnd,
        uID:    id,
        ..Default::default()
    };
    let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
}

unsafe fn update_tray_tip(hwnd: HWND, id: u32, tip: &str) -> windows::core::Result<()> {
    let mut tip_buf = [0u16; 128];
    let text: Vec<u16> = tip.encode_utf16().collect();
    tip_buf[..text.len().min(127)].copy_from_slice(&text[..text.len().min(127)]);
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd:   hwnd,
        uID:    id,
        uFlags: NIF_TIP,
        szTip:  tip_buf,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_MODIFY, &mut nid).ok()
}

unsafe fn show_balloon(hwnd: HWND, id: u32, title: &str, body: &str) -> windows::core::Result<()> {
    let mut info_title = [0u16; 64];
    let t: Vec<u16> = title.encode_utf16().collect();
    info_title[..t.len().min(63)].copy_from_slice(&t[..t.len().min(63)]);

    let mut info = [0u16; 256];
    let b: Vec<u16> = body.encode_utf16().collect();
    info[..b.len().min(255)].copy_from_slice(&b[..b.len().min(255)]);

    let mut nid = NOTIFYICONDATAW {
        cbSize:       std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd:         hwnd,
        uID:          id,
        uFlags:       NIF_INFO,
        szInfoTitle:  info_title,
        szInfo:       info,
        dwInfoFlags:  NIIF_NOSOUND,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_MODIFY, &mut nid).ok()
}

unsafe fn is_fullscreen_foreground(overlay_hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, SM_CXSCREEN, SM_CYSCREEN,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;

    let fg = GetForegroundWindow();
    if fg == overlay_hwnd || fg.0.is_null() {
        return false;
    }
    let mut r = RECT::default();
    if GetWindowRect(fg, &mut r).is_err() {
        return false;
    }
    let sw = GetSystemMetrics(SM_CXSCREEN);
    let sh = GetSystemMetrics(SM_CYSCREEN);
    let w  = r.right - r.left;
    let h  = r.bottom - r.top;
    w >= sw && h >= sh
}

fn other_audio_peak(own_pid: u32) -> f32 {
    let proc_map = build_process_snapshot();
    unsafe { detect_audio_activity(own_pid, &proc_map).0 }
}

fn build_process_snapshot() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return map;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let nul  = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                let raw  = String::from_utf16_lossy(&entry.szExeFile[..nul]);
                let name = if raw.to_ascii_lowercase().ends_with(".exe") {
                    raw[..raw.len().saturating_sub(4)].to_string()
                } else { raw };
                map.insert(entry.th32ProcessID, name);
                if Process32NextW(snap, &mut entry).is_err() { break; }
            }
        }
        let _ = CloseHandle(snap);
    }
    map
}

unsafe fn detect_audio_activity(
    own_pid:  u32,
    proc_map: &HashMap<u32, String>,
) -> (f32, Option<String>) {
    let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
        Ok(e) => e, Err(_) => return (0.0, None),
    };
    let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
        Ok(d) => d, Err(_) => return (0.0, None),
    };
    let mgr: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
        Ok(m) => m, Err(_) => return (0.0, None),
    };
    let ses_enum: IAudioSessionEnumerator = match mgr.GetSessionEnumerator() {
        Ok(e) => e, Err(_) => return (0.0, None),
    };
    let count: i32 = ses_enum.GetCount().unwrap_or(0);

    let mut max_peak    = 0.0f32;
    let mut active_pid:   Option<u32> = None; // sesión activa (sonando ahora)
    let mut inactive_pid: Option<u32> = None; // sesión inactiva (alt+tab, pausada)

    for i in 0..count {
        let ctrl  = match ses_enum.GetSession(i)         { Ok(c) => c, Err(_) => continue };
        let ctrl2: IAudioSessionControl2 = match ctrl.cast() { Ok(c) => c, Err(_) => continue };
        let pid   = ctrl2.GetProcessId().unwrap_or(own_pid);
        if pid == own_pid || pid == 0 { continue }

        // Sesiones Expired = proceso cerró su sesión de audio → ignorar
        let state = ctrl.GetState().unwrap_or(AudioSessionStateExpired);
        if state == AudioSessionStateExpired { continue }

        let meter: IAudioMeterInformation = match ctrl.cast() { Ok(m) => m, Err(_) => continue };
        let peak: f32 = meter.GetPeakValue().unwrap_or(0.0);

        if state == AudioSessionStateActive {
            // Activo: candidato principal para duck y para mostrar nombre
            if peak >= max_peak { max_peak = peak; active_pid = Some(pid); }
        } else if inactive_pid.is_none() {
            // Inactivo (alt+tab): sesión existe pero silenciada → fallback para nombre
            inactive_pid = Some(pid);
        }
    }

    // Duck usa solo sesiones activas con peak real
    // Nombre del juego: activo primero, si no hay → inactivo (alt+tab)
    let game_pid  = active_pid.or(inactive_pid);
    let game_name = game_pid.and_then(|pid| proc_map.get(&pid).cloned());

    (max_peak, game_name)
}

fn wide_truncated(s: &str, max: usize) -> Vec<u16> {
    if s.chars().count() <= max {
        s.encode_utf16().collect()
    } else {
        let head: String = s.chars().take(max - 1).collect();
        format!("{head}…").encode_utf16().collect()
    }
}
