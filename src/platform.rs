use lol_companion::overlay::{Rect, should_show};
use std::{
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::ClientToScreen,
    UI::WindowsAndMessaging::{
        FindWindowW, GWL_EXSTYLE, GetClientRect, GetForegroundWindow, GetWindowLongPtrW,
        HWND_TOPMOST, IsIconic, IsWindow, IsWindowVisible, LWA_ALPHA, SW_HIDE, SW_SHOWNOACTIVATE,
        SWP_NOACTIVATE, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    },
};

pub struct Shared {
    pub enabled: AtomicBool,
    // 0 = no game, 1 = not focused, 2 = visible, 3 = error
    pub status: AtomicU8,
}
impl Shared {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            status: AtomicU8::new(0),
        }
    }
}

pub fn start(overlay: isize, shared: Arc<Shared>) {
    thread::spawn(move || {
        let overlay = overlay as HWND;
        // SAFETY: The handle belongs to the overlay window created in this process.
        // The flags change only its extended style; Win32 accepts these calls across threads.
        unsafe {
            let style = GetWindowLongPtrW(overlay, GWL_EXSTYLE);
            SetWindowLongPtrW(
                overlay,
                GWL_EXSTYLE,
                style
                    | isize::try_from(
                        WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                    )
                    .unwrap_or_default(),
            );
            SetLayeredWindowAttributes(overlay, 0, 255, LWA_ALPHA);
        }
        let title: Vec<u16> = "League of Legends (TM) Client\0".encode_utf16().collect();
        // GPUI creates this window visible so its renderer can initialize.
        // Hide it on the first poll unless the game is focused.
        let mut shown = true;
        let mut last = None;
        loop {
            // SAFETY: Handles are only passed to Win32; buffers are initialized and valid.
            // Each result is checked before its coordinates are used. The window may disappear
            // at any time, in which case the next iteration hides the overlay.
            unsafe {
                if IsWindow(overlay) == 0 {
                    break;
                }
                let game = FindWindowW(ptr::null(), title.as_ptr());
                let mut rect = RECT::default();
                let mut origin = POINT::default();
                let valid = !game.is_null()
                    && IsWindowVisible(game) != 0
                    && IsIconic(game) == 0
                    && GetClientRect(game, &raw mut rect) != 0
                    && ClientToScreen(game, &raw mut origin) != 0;
                let bounds = valid.then_some(Rect {
                    x: origin.x,
                    y: origin.y,
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                });
                let focused = valid && GetForegroundWindow() == game;
                let visible = should_show(shared.enabled.load(Ordering::Relaxed), focused, bounds);
                let status = if game.is_null() {
                    0
                } else if visible {
                    2
                } else {
                    1
                };
                if let Some(b) = bounds.filter(|_| visible) {
                    if !shown || last != Some(b) {
                        if SetWindowPos(
                            overlay,
                            HWND_TOPMOST,
                            b.x,
                            b.y,
                            b.width,
                            b.height,
                            SWP_NOACTIVATE,
                        ) == 0
                        {
                            shared.status.store(3, Ordering::Relaxed);
                            ShowWindow(overlay, SW_HIDE);
                            shown = false;
                            last = None;
                            thread::sleep(Duration::from_millis(250));
                            continue;
                        }
                        ShowWindow(overlay, SW_SHOWNOACTIVATE);
                        last = Some(b);
                        shown = true;
                    }
                } else if shown {
                    ShowWindow(overlay, SW_HIDE);
                    shown = false;
                    last = None;
                }
                shared.status.store(status, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(250));
        }
    });
}
