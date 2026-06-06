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
            eConsole, eRender, AudioSessionStateActive, AudioSessionStateExpired,
            Endpoints::IAudioMeterInformation, IAudioSessionControl2, IAudioSessionEnumerator,
            IAudioSessionManager2, IMMDeviceEnumerator, MMDeviceEnumerator,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
            Console::GetConsoleWindow,
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            LibraryLoader::GetModuleHandleW,
            Threading::AttachThreadInput,
        },
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_NOSOUND, NIM_ADD,
                NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::*,
        },
    },
};

use crate::audio::{PlayerCommand, PlayerState, PlayerStatus};
use crate::config::{Config, OverlayMode, OverlayPosition, OverlayStyle};

const OW: i32 = 380;
const OH: i32 = 145;
const OH_COMPACT: i32 = 52;
const ACCENT_W: i32 = 3;
const PAD_L: i32 = ACCENT_W + 10;
const PAD_R: i32 = 8;

const VU_BARS: usize = 9;
const VU_BAR_W: i32 = 12;
const VU_BAR_GAP: i32 = 3;
const VU_Y: i32 = 114;
const VU_H: i32 = 20;
const VOL_BAR_W: i32 = 72;

const PEAK_HOLD_MS: u128 = 1500;
const PEAK_DECAY_TICK: f32 = 0.020;
const DUCK_THRESHOLD: f32 = 0.02;
const UNDUCK_DELAY_MS: u128 = 2000;

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(((b as u32) << 16) | ((g as u32) << 8) | (r as u32))
}

const C_BG: COLORREF = rgb(0x14, 0x14, 0x14);
const C_ACCENT: COLORREF = rgb(0x44, 0xCC, 0x33);
const C_BRAND: COLORREF = rgb(0xE8, 0xE8, 0xE8);
const C_SEPARATOR: COLORREF = rgb(0x2A, 0x2A, 0x2A);
const C_STATION: COLORREF = rgb(0x44, 0xCC, 0x44);
const C_SHOW: COLORREF = rgb(0xCC, 0xA0, 0x30);
const C_TITLE: COLORREF = rgb(0xF0, 0xF0, 0xF0);
const C_RECENT: COLORREF = rgb(0xFF, 0xFF, 0xFF);
const C_ST_OK: COLORREF = rgb(0x44, 0xCC, 0x33);
const C_ST_BUF: COLORREF = rgb(0xCC, 0x99, 0x00);
const C_ST_RECO: COLORREF = rgb(0xFF, 0x80, 0x00);
const C_VU_BG: COLORREF = rgb(0x28, 0x28, 0x28);
const C_VU_LOW: COLORREF = rgb(0x33, 0xAA, 0x33);
const C_VU_MED: COLORREF = rgb(0xAA, 0xCC, 0x00);
const C_VU_HIGH: COLORREF = rgb(0xFF, 0x44, 0x00);
const C_VOL_FG: COLORREF = rgb(0x44, 0xAA, 0x55);

#[derive(Clone, Copy)]
enum OStatus {
    Playing,
    Buffering(f32),
    Reconnecting(u32),
}

struct State {
    station: String,
    show: String,
    title: String,
    level_db: f32,
    ostatus: OStatus,
    peak_ratio: f32,
    volume: f32,
    bitrate_kbps: Option<u16>,
    recent: Vec<String>,
    overlay_position: crate::config::OverlayPosition,
    overlay_style: crate::config::OverlayStyle,
    duck_enabled: bool,
}

static MEDIA_VK_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<u32>> =
    std::sync::OnceLock::new();

pub fn spawn(
    state_rx: watch::Receiver<PlayerState>,
    config_rx: watch::Receiver<Config>,
    cmd_tx: mpsc::Sender<PlayerCommand>,
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

unsafe fn run(
    mut rx: watch::Receiver<PlayerState>,
    mut config_rx: watch::Receiver<Config>,
    cmd_tx: mpsc::Sender<PlayerCommand>,
) -> windows::core::Result<()> {
    let own_pid = std::process::id();
    {
        std::thread::Builder::new()
            .name("wasapi-monitor".into())
            .spawn(move || unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                loop {
                    let proc_map = build_process_snapshot();
                    let (_, detected) = detect_audio_activity(own_pid, &proc_map);
                    let raw = detected.unwrap_or_default();
                    crate::game_detect::set(if raw.is_empty() { None } else { Some(raw) });
                    std::thread::sleep(Duration::from_millis(500));
                }
            })
            .expect("wasapi-monitor thread");
    }

    let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);
    let class = w!("ReverbicOverlay");

    let shared = Arc::new(Mutex::new(State {
        station: String::new(),
        show: String::new(),
        title: String::new(),
        level_db: -60.0,
        ostatus: OStatus::Playing,
        peak_ratio: 0.0,
        volume: 1.0,
        bitrate_kbps: None,
        recent: Vec::new(),
        overlay_position: config_rx.borrow().overlay_position,
        overlay_style: config_rx.borrow().overlay_style,
        duck_enabled: config_rx.borrow().duck_enabled,
    }));
    let raw = Arc::into_raw(Arc::clone(&shared));

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance,
        lpszClassName: class,
        ..Default::default()
    };
    RegisterClassExW(&wc);

    let init_style = config_rx.borrow().overlay_style;
    let init_oh = match init_style {
        OverlayStyle::Compact => OH_COMPACT,
        OverlayStyle::Full => OH,
    };
    let (init_x, init_y) = overlay_coords(GetConsoleWindow(), config_rx.borrow().overlay_position, init_oh);
    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
        class,
        w!("Reverbic"),
        WS_POPUP,
        init_x,
        init_y,
        OW,
        init_oh,
        None,
        None,
        hinstance,
        Some(raw as *const _),
    )?;

    let init_alpha: u8 = (config_rx.borrow().overlay_alpha.min(100) as u32 * 255 / 100) as u8;
    SetLayeredWindowAttributes(hwnd, COLORREF(0), init_alpha, LWA_ALPHA)?;

    let mut msg = MSG::default();
    let mut visible = false;
    let mut peak_ratio = 0_f32;
    let mut peak_held_at = Instant::now();
    let mut cfg = config_rx.borrow().clone();
    let mut playing = false;
    let mut last_title = String::new();
    let mut prev_position = cfg.overlay_position;
    let mut prev_style = cfg.overlay_style;

    let (media_tx, media_rx) = std::sync::mpsc::sync_channel::<u32>(4);
    let _ = MEDIA_VK_TX.set(media_tx);

    let mut hook: HHOOK = HHOOK::default();
    let mut tray_added: bool = false;
    let tray_id: u32 = 1001;
    let wm_tray = WM_APP + 1;

    let mut is_ducked: bool = false;
    let mut pre_duck_vol: f32 = 1.0;
    let mut quiet_since: Option<Instant> = None;
    let mut next_duck_check: Instant = Instant::now();
    if cfg.media_keys {
        hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0).unwrap_or_default();
    }
    if cfg.tray_icon {
        tray_added = add_tray_icon(hwnd, tray_id, wm_tray).is_ok();
    }

    loop {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                if !hook.0.is_null() {
                    let _ = UnhookWindowsHookEx(hook);
                }
                if tray_added {
                    remove_tray_icon(hwnd, tray_id);
                }
                return Ok(());
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        while let Ok(vk) = media_rx.try_recv() {
            let ps = rx.borrow().clone();
            let cmd = match vk {
                0xB3 => match ps.status {
                    PlayerStatus::Playing => Some(PlayerCommand::Pause),
                    PlayerStatus::Paused => Some(PlayerCommand::Resume),
                    _ => None,
                },
                0xB2 => Some(PlayerCommand::Stop),
                _ => None,
            };
            if let Some(c) = cmd {
                let _ = cmd_tx.try_send(c);
            }
        }
        if config_rx.has_changed().unwrap_or(false) {
            cfg = config_rx.borrow_and_update().clone();
            if cfg.media_keys && hook.0.is_null() {
                hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0)
                    .unwrap_or_default();
            } else if !cfg.media_keys && !hook.0.is_null() {
                let _ = UnhookWindowsHookEx(hook);
                hook = HHOOK::default();
            }
            if cfg.tray_icon && !tray_added {
                tray_added = add_tray_icon(hwnd, tray_id, wm_tray).is_ok();
            } else if !cfg.tray_icon && tray_added {
                remove_tray_icon(hwnd, tray_id);
                tray_added = false;
            }
            let new_alpha: u8 = (cfg.overlay_alpha.min(100) as u32 * 255 / 100) as u8;
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), new_alpha, LWA_ALPHA);
            if cfg.overlay_style != prev_style || cfg.overlay_position != prev_position {
                prev_style = cfg.overlay_style;
                prev_position = cfg.overlay_position;
                let new_h = match cfg.overlay_style {
                    OverlayStyle::Compact => OH_COMPACT,
                    OverlayStyle::Full => OH,
                };
                let (x, y) = overlay_coords(hwnd, cfg.overlay_position, new_h);
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    x,
                    y,
                    OW,
                    new_h,
                    SWP_NOACTIVATE,
                );
            }
            if let Ok(mut s) = shared.lock() {
                s.overlay_position = cfg.overlay_position;
                s.overlay_style = cfg.overlay_style;
            }
        }

        let mut need_repaint = false;
        if rx.has_changed().unwrap_or(false) {
            let ps = rx.borrow_and_update().clone();
            playing = matches!(
                ps.status,
                PlayerStatus::Playing | PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_)
            );

            let new_title = ps.title.clone().unwrap_or_default();
            let title_changed = new_title != last_title && !new_title.is_empty() && playing;

            if let Ok(mut s) = shared.lock() {
                s.station = ps
                    .station
                    .as_ref()
                    .map(|st| st.name.clone())
                    .unwrap_or_default();
                s.show = ps.api_show.clone().unwrap_or_default();
                s.title = new_title.clone();
                s.level_db = ps.level_db;
                s.volume = ps.volume;
                s.bitrate_kbps = ps.station.as_ref().and_then(|st| st.bitrate_kbps);
                s.recent = ps.recent_titles.iter().take(3).cloned().collect();
                s.duck_enabled = cfg.duck_enabled;
                s.ostatus = match ps.status {
                    PlayerStatus::Buffering(f) => OStatus::Buffering(f),
                    PlayerStatus::Reconnecting(n) => OStatus::Reconnecting(n),
                    _ => OStatus::Playing,
                };
                let cur = ((ps.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
                if cur >= peak_ratio {
                    peak_ratio = cur;
                    peak_held_at = Instant::now();
                }
                s.peak_ratio = peak_ratio;
            }
            if title_changed && cfg.notifications && tray_added {
                last_title = new_title.clone();
                let station = ps
                    .station
                    .as_ref()
                    .map(|s| s.name.as_str())
                    .unwrap_or("Reverbic");
                let _ = show_balloon(hwnd, tray_id, station, &new_title);
            }
            if tray_added {
                let tip = if !new_title.is_empty() {
                    format!("Reverbic — {new_title}")
                } else {
                    "Reverbic".to_string()
                };
                let _ = update_tray_tip(hwnd, tray_id, &tip);
            }
        }
        let should_show = match cfg.overlay_mode {
            OverlayMode::Hidden => false,
            OverlayMode::Always => playing,
            OverlayMode::WhenPlaying => playing,
            OverlayMode::Games => playing && is_fullscreen_foreground(hwnd),
        };

        if should_show != visible {
            visible = should_show;
            let _ = ShowWindow(
                hwnd,
                if should_show {
                    SW_SHOWNOACTIVATE
                } else {
                    SW_HIDE
                },
            );
        }
        if visible {
            need_repaint = true;
        }
        if cfg.duck_enabled && playing {
            if Instant::now() >= next_duck_check {
                next_duck_check = Instant::now() + Duration::from_millis(500);
                let peak = other_audio_peak(own_pid);
                let duck_target = cfg.duck_volume as f32 / 100.0;

                if peak > DUCK_THRESHOLD {
                    quiet_since = None;
                    if !is_ducked {
                        let current_vol = rx.borrow().volume;
                        if current_vol > duck_target + 0.01 {
                            pre_duck_vol = current_vol;
                            let _ = cmd_tx.try_send(PlayerCommand::SetVolume(duck_target));
                            is_ducked = true;
                        }
                    }
                } else if is_ducked {
                    match quiet_since {
                        None => quiet_since = Some(Instant::now()),
                        Some(t) if t.elapsed().as_millis() >= UNDUCK_DELAY_MS => {
                            let _ = cmd_tx.try_send(PlayerCommand::SetVolume(pre_duck_vol));
                            is_ducked = false;
                            quiet_since = None;
                        }
                        _ => {}
                    }
                }
            }
        } else if is_ducked {
            let _ = cmd_tx.try_send(PlayerCommand::SetVolume(pre_duck_vol));
            is_ducked = false;
            quiet_since = None;
        }
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
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let cs = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_DISPLAYCHANGE => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<State>;
            if !ptr.is_null() {
                if let Ok(s) = (*ptr).lock() {
                    let h = match s.overlay_style {
                        OverlayStyle::Compact => OH_COMPACT,
                        OverlayStyle::Full => OH,
                    };
                    let (x, y) = overlay_coords(hwnd, s.overlay_position, h);
                    let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y, OW, h, SWP_NOACTIVATE);
                }
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<State>;
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !ptr.is_null() {
                if let Ok(s) = (*ptr).lock() {
                    match s.overlay_style {
                        OverlayStyle::Compact => paint_compact(hdc, &s),
                        OverlayStyle::Full => paint(hdc, &s),
                    }
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
        x if x == WM_APP + 1 => {
            let event = (lparam.0 as u16) as u32;
            if event == WM_LBUTTONDBLCLK {
                let console = GetConsoleWindow();
                if !console.0.is_null() {
                    let fg = GetForegroundWindow();
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

unsafe fn paint(hdc: HDC, s: &State) {
    fill(
        hdc,
        RECT {
            left: 0,
            top: 0,
            right: OW,
            bottom: OH,
        },
        C_BG,
    );
    fill(
        hdc,
        RECT {
            left: 0,
            top: 0,
            right: ACCENT_W,
            bottom: OH,
        },
        C_ACCENT,
    );

    SetBkMode(hdc, TRANSPARENT);

    let f_brand = font(13, true);
    let f_small = font(11, false);
    let f_station = font(16, true);
    let f_detail = font(12, true);
    let f_recent = font(11, false);

    let prev = SelectObject(hdc, HGDIOBJ(f_brand.0));

    // ── header row ─────────────────────────────────────────────────
    SetTextColor(hdc, C_BRAND);
    let _ = TextOutW(hdc, PAD_L, 5, &wide("REVERBIC"));

    let clock = chrono::Local::now().format("%H:%M").to_string();
    let prev_align = SetTextAlign(hdc, TA_RIGHT);
    SetTextColor(hdc, C_BRAND);
    let _ = TextOutW(hdc, OW - PAD_R, 5, &wide(&clock));

    SelectObject(hdc, HGDIOBJ(f_small.0));
    let (status_str, status_color) = status_label(s.ostatus);
    SetTextColor(hdc, status_color);
    let _ = TextOutW(hdc, OW - PAD_R - 60, 5, &wide(&status_str));
    let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(prev_align));

    fill(
        hdc,
        RECT {
            left: PAD_L,
            top: 22,
            right: OW - PAD_R,
            bottom: 23,
        },
        C_SEPARATOR,
    );

    // ── station row ────────────────────────────────────────────────
    SelectObject(hdc, HGDIOBJ(f_station.0));
    SetTextColor(hdc, C_STATION);
    let _ = TextOutW(hdc, PAD_L, 27, &wide_truncated(&s.station, 34));

    if let Some(kbps) = s.bitrate_kbps {
        SetTextColor(hdc, C_BRAND);
        let prev_a = SetTextAlign(hdc, TA_RIGHT);
        let _ = TextOutW(hdc, OW - PAD_R, 30, &wide(&format!("{}k", kbps)));
        let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(prev_a));
    }

    fill(
        hdc,
        RECT {
            left: PAD_L,
            top: 46,
            right: OW - PAD_R,
            bottom: 47,
        },
        C_SEPARATOR,
    );

    // ── track info ─────────────────────────────────────────────────
    SelectObject(hdc, HGDIOBJ(f_detail.0));
    let has_show = !s.show.is_empty();
    if has_show {
        SetTextColor(hdc, C_SHOW);
        let _ = TextOutW(hdc, PAD_L, 50, &wide_truncated(&s.show, 46));
    }
    let y_title = if has_show { 64 } else { 50 };
    SetTextColor(hdc, C_TITLE);
    let title = if s.title.is_empty() {
        "—"
    } else {
        s.title.as_str()
    };
    let _ = TextOutW(hdc, PAD_L, y_title, &wide_truncated(title, 50));

    fill(
        hdc,
        RECT {
            left: PAD_L,
            top: 79,
            right: OW - PAD_R,
            bottom: 80,
        },
        C_SEPARATOR,
    );

    // ── recent tracks ──────────────────────────────────────────────
    SelectObject(hdc, HGDIOBJ(f_recent.0));
    SetTextColor(hdc, C_RECENT);
    for (i, track) in s.recent.iter().enumerate().skip(1).take(2) {
        let y = 83 + (i as i32 - 1) * 14;
        let _ = TextOutW(
            hdc,
            PAD_L,
            y,
            &wide_truncated(&format!("\u{21B3} {}", track), 52),
        );
    }

    fill(
        hdc,
        RECT {
            left: PAD_L,
            top: 110,
            right: OW - PAD_R,
            bottom: 111,
        },
        C_SEPARATOR,
    );

    // ── VU bars ────────────────────────────────────────────────────
    let base = ((s.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
    let t_ms = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    for i in 0..VU_BARS {
        let bx = PAD_L + (i as i32) * (VU_BAR_W + VU_BAR_GAP);
        fill(
            hdc,
            RECT {
                left: bx,
                top: VU_Y,
                right: bx + VU_BAR_W,
                bottom: VU_Y + VU_H,
            },
            C_VU_BG,
        );
        let phase = t_ms as f64 * 0.005 + (i as f64) * 0.65;
        let anim = (phase.sin() * 0.3 + 0.7) as f32;
        let level = (base * anim).clamp(0.0, 1.0);
        let min_h = if base > 0.02 { 2 } else { 0 };
        let bh = ((VU_H as f32 * level) as i32).max(min_h);
        if bh > 0 {
            fill(
                hdc,
                RECT {
                    left: bx,
                    top: VU_Y + VU_H - bh,
                    right: bx + VU_BAR_W,
                    bottom: VU_Y + VU_H,
                },
                vu_color(level),
            );
        }
    }

    // ── volume bar ─────────────────────────────────────────────────
    let vu_end_x = PAD_L + VU_BARS as i32 * (VU_BAR_W + VU_BAR_GAP) - VU_BAR_GAP;
    let vol_label_x = vu_end_x + 10;
    let vol_bar_x = vol_label_x + 28;
    let vol_bar_y = VU_Y + 6;

    SelectObject(hdc, HGDIOBJ(f_small.0));
    SetTextColor(hdc, C_BRAND);
    let _ = TextOutW(hdc, vol_label_x, VU_Y + 3, &wide("VOL"));

    fill(
        hdc,
        RECT {
            left: vol_bar_x,
            top: vol_bar_y,
            right: vol_bar_x + VOL_BAR_W,
            bottom: vol_bar_y + 8,
        },
        C_VU_BG,
    );
    let fill_w = (VOL_BAR_W as f32 * s.volume.clamp(0.0, 1.0)) as i32;
    if fill_w > 0 {
        fill(
            hdc,
            RECT {
                left: vol_bar_x,
                top: vol_bar_y,
                right: vol_bar_x + fill_w,
                bottom: vol_bar_y + 8,
            },
            C_VOL_FG,
        );
    }

    SetTextColor(hdc, C_TITLE);
    let vol_pct = (s.volume * 100.0).round() as u32;
    let _ = TextOutW(
        hdc,
        vol_bar_x + VOL_BAR_W + 5,
        VU_Y + 3,
        &wide(&format!("{}%", vol_pct)),
    );

    let duck_color = if s.duck_enabled {
        C_ACCENT
    } else {
        rgb(0x3A, 0x3A, 0x3A)
    };
    SetTextColor(hdc, duck_color);
    let prev_a = SetTextAlign(hdc, TA_RIGHT);
    let _ = TextOutW(hdc, OW - PAD_R, VU_Y + 3, &wide("DUCK"));
    let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(prev_a));

    SelectObject(hdc, prev);
    let _ = DeleteObject(HGDIOBJ(f_brand.0));
    let _ = DeleteObject(HGDIOBJ(f_small.0));
    let _ = DeleteObject(HGDIOBJ(f_station.0));
    let _ = DeleteObject(HGDIOBJ(f_detail.0));
    let _ = DeleteObject(HGDIOBJ(f_recent.0));
}

unsafe fn paint_compact(hdc: HDC, s: &State) {
    fill(
        hdc,
        RECT { left: 0, top: 0, right: OW, bottom: OH_COMPACT },
        C_BG,
    );
    fill(
        hdc,
        RECT { left: 0, top: 0, right: ACCENT_W, bottom: OH_COMPACT },
        C_ACCENT,
    );

    SetBkMode(hdc, TRANSPARENT);

    let f_station = font(14, true);
    let f_detail = font(12, false);

    let prev = SelectObject(hdc, HGDIOBJ(f_station.0));

    // ── row 1: status dot + station ──────────────────────────────────
    let (status_str, status_color) = status_label(s.ostatus);
    SetTextColor(hdc, status_color);
    let _ = TextOutW(hdc, PAD_L, 6, &wide(&status_str));

    let dot_w = 14_i32;
    SetTextColor(hdc, C_STATION);
    let _ = TextOutW(
        hdc,
        PAD_L + dot_w,
        5,
        &wide_truncated(&s.station, 28),
    );

    // clock right-aligned
    SetTextColor(hdc, C_BRAND);
    let prev_align = SetTextAlign(hdc, TA_RIGHT);
    let clock = chrono::Local::now().format("%H:%M").to_string();
    let _ = TextOutW(hdc, OW - PAD_R, 5, &wide(&clock));
    let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(prev_align));

    // ── separator ────────────────────────────────────────────────────
    fill(
        hdc,
        RECT { left: PAD_L, top: 24, right: OW - PAD_R, bottom: 25 },
        C_SEPARATOR,
    );

    // ── row 2: track title ───────────────────────────────────────────
    SelectObject(hdc, HGDIOBJ(f_detail.0));
    SetTextColor(hdc, C_TITLE);
    let title = if s.title.is_empty() { "—" } else { s.title.as_str() };
    let _ = TextOutW(hdc, PAD_L, 30, &wide_truncated(title, 50));

    SelectObject(hdc, prev);
    let _ = DeleteObject(HGDIOBJ(f_station.0));
    let _ = DeleteObject(HGDIOBJ(f_detail.0));
}

fn status_label(os: OStatus) -> (String, COLORREF) {
    match os {
        OStatus::Playing => ("●".into(), C_ST_OK),
        OStatus::Buffering(f) => (format!("◌ {:.0}%", f * 100.0), C_ST_BUF),
        OStatus::Reconnecting(n) => (format!("↻ x{n}"), C_ST_RECO),
    }
}

fn vu_color(ratio: f32) -> COLORREF {
    if ratio < 0.5 {
        C_VU_LOW
    } else if ratio < 0.8 {
        C_VU_MED
    } else {
        C_VU_HIGH
    }
}

unsafe fn fill(hdc: HDC, r: RECT, color: COLORREF) {
    let b = CreateSolidBrush(color);
    FillRect(hdc, &r, b);
    let _ = DeleteObject(HGDIOBJ(b.0));
}

unsafe fn font(size: i32, bold: bool) -> HFONT {
    CreateFontW(
        size,
        0,
        0,
        0,
        if bold { 700 } else { 400 },
        0,
        0,
        0,
        1,
        0,
        0,
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
    let hmodule = GetModuleHandleW(None)?;
    let icon = LoadIconW(hmodule, PCWSTR(std::ptr::dangling::<u16>())).unwrap_or_else(|_| {
        LoadIconW(HINSTANCE::default(), IDI_APPLICATION)
            .expect("IDI_APPLICATION siempre disponible")
    });
    let mut tip = [0u16; 128];
    let text: Vec<u16> = "Reverbic".encode_utf16().collect();
    tip[..text.len().min(127)].copy_from_slice(&text[..text.len().min(127)]);

    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: id,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: callback_msg,
        hIcon: icon,
        szTip: tip,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_ADD, &nid).ok()
}

unsafe fn remove_tray_icon(hwnd: HWND, id: u32) {
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: id,
        ..Default::default()
    };
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe fn update_tray_tip(hwnd: HWND, id: u32, tip: &str) -> windows::core::Result<()> {
    let mut tip_buf = [0u16; 128];
    let text: Vec<u16> = tip.encode_utf16().collect();
    tip_buf[..text.len().min(127)].copy_from_slice(&text[..text.len().min(127)]);
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: id,
        uFlags: NIF_TIP,
        szTip: tip_buf,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_MODIFY, &nid).ok()
}

unsafe fn show_balloon(hwnd: HWND, id: u32, title: &str, body: &str) -> windows::core::Result<()> {
    let mut info_title = [0u16; 64];
    let t: Vec<u16> = title.encode_utf16().collect();
    info_title[..t.len().min(63)].copy_from_slice(&t[..t.len().min(63)]);

    let mut info = [0u16; 256];
    let b: Vec<u16> = body.encode_utf16().collect();
    info[..b.len().min(255)].copy_from_slice(&b[..b.len().min(255)]);

    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: id,
        uFlags: NIF_INFO,
        szInfoTitle: info_title,
        szInfo: info,
        dwInfoFlags: NIIF_NOSOUND,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_MODIFY, &nid).ok()
}

unsafe fn is_fullscreen_foreground(overlay_hwnd: HWND) -> bool {
    let fg = GetForegroundWindow();
    if fg == overlay_hwnd || fg.0.is_null() {
        return false;
    }
    let mut class_buf = [0u16; 128];
    let class_len = GetClassNameW(fg, &mut class_buf) as usize;
    if class_len > 0 {
        let class = String::from_utf16_lossy(&class_buf[..class_len]);
        match class.as_str() {
            "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" => return false,
            _ => {}
        }
    }
    let mut r = RECT::default();
    if GetWindowRect(fg, &mut r).is_err() {
        return false;
    }
    let monitor = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let (sw, sh) = if GetMonitorInfoW(monitor, &mut info).as_bool() {
        let mr = info.rcMonitor;
        (mr.right - mr.left, mr.bottom - mr.top)
    } else {
        (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
    };
    let w = r.right - r.left;
    let h = r.bottom - r.top;
    w >= sw && h >= sh
}

unsafe fn overlay_coords(hwnd: HWND, pos: OverlayPosition, oh: i32) -> (i32, i32) {
    const MARGIN: i32 = 16;
    let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let (sw, sh, ox, oy) = if GetMonitorInfoW(monitor, &mut info).as_bool() {
        let r = info.rcMonitor;
        (r.right - r.left, r.bottom - r.top, r.left, r.top)
    } else {
        (
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            0,
            0,
        )
    };
    match pos {
        OverlayPosition::TopLeft => (ox + MARGIN, oy + MARGIN),
        OverlayPosition::TopRight => (ox + sw - OW - MARGIN, oy + MARGIN),
        OverlayPosition::BottomLeft => (ox + MARGIN, oy + sh - oh - MARGIN),
        OverlayPosition::BottomRight => (ox + sw - OW - MARGIN, oy + sh - oh - MARGIN),
    }
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
                let nul = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                let raw = String::from_utf16_lossy(&entry.szExeFile[..nul]);
                let name = if raw.to_ascii_lowercase().ends_with(".exe") {
                    raw[..raw.len().saturating_sub(4)].to_string()
                } else {
                    raw
                };
                map.insert(entry.th32ProcessID, name);
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    map
}

unsafe fn detect_audio_activity(
    own_pid: u32,
    proc_map: &HashMap<u32, String>,
) -> (f32, Option<String>) {
    let enumerator: IMMDeviceEnumerator =
        match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
            Ok(e) => e,
            Err(_) => return (0.0, None),
        };
    let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
        Ok(d) => d,
        Err(_) => return (0.0, None),
    };
    let mgr: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
        Ok(m) => m,
        Err(_) => return (0.0, None),
    };
    let ses_enum: IAudioSessionEnumerator = match mgr.GetSessionEnumerator() {
        Ok(e) => e,
        Err(_) => return (0.0, None),
    };
    let count: i32 = ses_enum.GetCount().unwrap_or(0);

    let mut max_peak = 0.0f32;
    let mut active_pid: Option<u32> = None;
    let mut inactive_pid: Option<u32> = None;

    for i in 0..count {
        let ctrl = match ses_enum.GetSession(i) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let ctrl2: IAudioSessionControl2 = match ctrl.cast() {
            Ok(c) => c,
            Err(_) => continue,
        };
        let pid = ctrl2.GetProcessId().unwrap_or(own_pid);
        if pid == own_pid || pid == 0 {
            continue;
        }
        let state = ctrl.GetState().unwrap_or(AudioSessionStateExpired);
        if state == AudioSessionStateExpired {
            continue;
        }

        let meter: IAudioMeterInformation = match ctrl.cast() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let peak: f32 = meter.GetPeakValue().unwrap_or(0.0);

        if state == AudioSessionStateActive {
            if peak >= max_peak {
                max_peak = peak;
                active_pid = Some(pid);
            }
        } else if inactive_pid.is_none() {
            inactive_pid = Some(pid);
        }
    }
    let game_pid = active_pid.or(inactive_pid);
    let game_name = game_pid.and_then(|pid| proc_map.get(&pid).cloned());

    (max_peak, game_name)
}

fn wide_truncated(s: &str, max: usize) -> Vec<u16> {
    if s.chars().count() <= max {
        s.encode_utf16().collect()
    } else {
        let head: String = s.chars().take(max - 1).collect();
        format!("{head}\u{2026}").encode_utf16().collect()
    }
}
