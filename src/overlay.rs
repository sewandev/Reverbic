#![cfg(target_os = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

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

const OW: i32 = 300;
const OH: i32 = 60;

struct State {
    station: String,
    title:   String,
}

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

unsafe fn run(mut rx: watch::Receiver<PlayerState>) -> windows::core::Result<()> {
    let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);
    let class     = w!("ReverbicOverlay");

    let shared = Arc::new(Mutex::new(State { station: String::new(), title: String::new() }));
    let raw    = Arc::into_raw(Arc::clone(&shared));

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
        16,
        16,
        OW, OH,
        None, None, hinstance,
        Some(raw as *const _),
    )?;

    SetLayeredWindowAttributes(hwnd, COLORREF(0), 230, LWA_ALPHA)?;

    let mut msg     = MSG::default();
    let mut visible = false;

    loop {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                return Ok(());
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if rx.has_changed().unwrap_or(false) {
            let ps      = rx.borrow_and_update().clone();
            let playing = matches!(ps.status, PlayerStatus::Playing | PlayerStatus::Buffering(_));

            if let Ok(mut s) = shared.lock() {
                s.station = ps.station.map(|st| st.name).unwrap_or_default();
                s.title   = ps.title.unwrap_or_default();
            }

            if playing != visible {
                visible = playing;
                let _ = ShowWindow(hwnd, if playing { SW_SHOWNOACTIVATE } else { SW_HIDE });
            }
            if playing {
                let _ = InvalidateRect(hwnd, None, TRUE);
                let _ = UpdateWindow(hwnd);
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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

unsafe fn paint(hdc: HDC, state: &State) {
    let full = RECT { left: 0, top: 0, right: OW, bottom: OH };

    let bg = CreateSolidBrush(COLORREF(0x00141414));
    FillRect(hdc, &full, bg);
    let _ = DeleteObject(HGDIOBJ(bg.0));

    let accent = CreateSolidBrush(COLORREF(0x0033AA44));
    FillRect(hdc, &RECT { left: 0, top: 0, right: 3, bottom: OH }, accent);
    let _ = DeleteObject(HGDIOBJ(accent.0));

    SetBkMode(hdc, TRANSPARENT);

    let f_station = font(15, true);
    let prev = SelectObject(hdc, HGDIOBJ(f_station.0));
    SetTextColor(hdc, COLORREF(0x0033CC44));
    let sw = wide_truncated(&state.station, 33);
    let _ = TextOutW(hdc, 12, 8, &sw);

    let f_track = font(13, false);
    SelectObject(hdc, HGDIOBJ(f_track.0));
    SetTextColor(hdc, COLORREF(0x00AAAAAA));
    let title = if state.title.is_empty() { "—" } else { state.title.as_str() };
    let tw = wide_truncated(title, 38);
    let _ = TextOutW(hdc, 12, 36, &tw);

    SelectObject(hdc, prev);
    let _ = DeleteObject(HGDIOBJ(f_station.0));
    let _ = DeleteObject(HGDIOBJ(f_track.0));
}

unsafe fn font(size: i32, bold: bool) -> HFONT {
    CreateFontW(
        size, 0, 0, 0,
        if bold { 700 } else { 400 },
        0, 0, 0,
        1,  // DEFAULT_CHARSET
        0,  // OUT_DEFAULT_PRECIS
        0,  // CLIP_DEFAULT_PRECIS
        5,  // CLEARTYPE_QUALITY
        0,  // DEFAULT_PITCH | FF_DONTCARE
        w!("Segoe UI"),
    )
}

fn wide_truncated(s: &str, max: usize) -> Vec<u16> {
    if s.chars().count() <= max {
        s.encode_utf16().collect()
    } else {
        let head: String = s.chars().take(max - 1).collect();
        format!("{head}…").encode_utf16().collect()
    }
}
