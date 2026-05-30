#![cfg(target_os = "windows")]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::watch;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

use crate::audio::{PlayerState, PlayerStatus};

const OW: i32 = 320;
const OH: i32 = 90;
const ACCENT_W: i32 = 3;
const PAD_L:    i32 = ACCENT_W + 10;
const PAD_R:    i32 = 8;

const PEAK_HOLD_MS:     u128 = 1500;
const PEAK_DECAY_TICK:  f32  = 0.020; // por tick de 50ms → full-scale en ~2.5s

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
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn spawn(state_rx: watch::Receiver<PlayerState>) {
    std::thread::Builder::new()
        .name("overlay".into())
        .spawn(move || unsafe {
            if let Err(e) = run(state_rx) {
                tracing::error!("overlay: {e}");
            }
        })
        .expect("overlay thread");
}

// ── Win32 thread ──────────────────────────────────────────────────────────────

unsafe fn run(mut rx: watch::Receiver<PlayerState>) -> windows::core::Result<()> {
    let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);
    let class     = w!("ReverbicOverlay");

    let shared = Arc::new(Mutex::new(State {
        station:    String::new(),
        show:       String::new(),
        title:      String::new(),
        level_db:   -60.0,
        ostatus:    OStatus::Playing,
        peak_ratio: 0.0,
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

    SetLayeredWindowAttributes(hwnd, COLORREF(0), 230, LWA_ALPHA)?;

    let mut msg          = MSG::default();
    let mut visible      = false;
    let mut peak_ratio   = 0_f32;
    let mut peak_held_at = Instant::now();

    loop {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                return Ok(());
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let mut need_repaint = false;

        // ── New player state ──────────────────────────────────────
        if rx.has_changed().unwrap_or(false) {
            let ps = rx.borrow_and_update().clone();
            let playing = matches!(
                ps.status,
                PlayerStatus::Playing | PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_)
            );

            if let Ok(mut s) = shared.lock() {
                s.station  = ps.station.map(|st| st.name).unwrap_or_default();
                s.show     = ps.api_show.unwrap_or_default();
                s.title    = ps.title.unwrap_or_default();
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

            if playing != visible {
                visible = playing;
                let _ = ShowWindow(hwnd, if playing { SW_SHOWNOACTIVATE } else { SW_HIDE });
            }
            if playing {
                need_repaint = true;
            }
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

    // ── VU meter + peak hold ──────────────────────────────────────
    let vu_x1 = PAD_L;
    let vu_x2 = OW - PAD_R;
    fill(hdc, RECT { left: vu_x1, top: 79, right: vu_x2, bottom: 83 }, C_VU_BG);

    let ratio = ((s.level_db + 60.0) / 60.0).clamp(0.0, 1.0);
    let bar_w = ((vu_x2 - vu_x1) as f32 * ratio) as i32;
    if bar_w > 0 {
        fill(hdc, RECT { left: vu_x1, top: 79, right: vu_x1 + bar_w, bottom: 83 }, vu_color(ratio));
    }

    // Peak hold: marca vertical de 2px del color del nivel pico
    if s.peak_ratio > 0.02 {
        let px = (vu_x1 + ((vu_x2 - vu_x1) as f32 * s.peak_ratio) as i32).min(vu_x2 - 2);
        fill(hdc, RECT { left: px, top: 79, right: px + 2, bottom: 83 }, vu_color(s.peak_ratio));
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

fn wide_truncated(s: &str, max: usize) -> Vec<u16> {
    if s.chars().count() <= max {
        s.encode_utf16().collect()
    } else {
        let head: String = s.chars().take(max - 1).collect();
        format!("{head}…").encode_utf16().collect()
    }
}
