#[cfg(windows)]
mod platform;

#[cfg(windows)]
mod windows_app {
    use crate::platform::{self, Shared};
    use gpui::{
        App, Application, Bounds, Context, IntoElement, ParentElement, Render, Styled, Timer,
        Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, div,
        prelude::*, px, rgb, rgba, size,
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::{
        sync::{Arc, atomic::Ordering},
        time::Duration,
    };

    struct Overlay;
    impl Render for Overlay {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .p_6()
                .text_xl()
                .text_color(rgb(0x00ff_ffff))
                .child(
                    div()
                        .bg(rgba(0x1018_20cc))
                        .rounded_md()
                        .p_3()
                        .child("Hello, world!"),
                )
        }
    }
    struct Dashboard {
        shared: Arc<Shared>,
    }
    impl Render for Dashboard {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let enabled = self.shared.enabled.load(Ordering::Relaxed);
            let status = match self.shared.status.load(Ordering::Relaxed) {
                0 => "Waiting for League",
                1 => "League detected; overlay hidden",
                2 => "League detected; overlay visible",
                _ => "Could not position overlay",
            };
            div()
                .size_full()
                .bg(rgb(0x0010_1820))
                .text_color(rgb(0x00f4_f6fa))
                .p_8()
                .flex()
                .flex_col()
                .gap_4()
                .child(div().text_2xl().child("League Companion"))
                .child(status)
                .child(
                    div()
                        .id("toggle")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(rgb(0x0031_5d8a))
                        .cursor_pointer()
                        .child(if enabled {
                            "Disable overlay"
                        } else {
                            "Enable overlay"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            let enabled = this.shared.enabled.load(Ordering::Relaxed);
                            this.shared.enabled.store(!enabled, Ordering::Relaxed);
                            cx.notify();
                        })),
                )
        }
    }
    pub fn run() {
        // SAFETY: This process requests per-monitor DPI awareness before it creates any windows.
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
                windows_sys::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            );
        }
        Application::new().run(|cx: &mut App| {
            let shared = Arc::new(Shared::new());
            let overlay_shared = Arc::clone(&shared);
            let overlay = cx.open_window(
                WindowOptions {
                    kind: WindowKind::PopUp,
                    window_background: WindowBackgroundAppearance::Transparent,
                    focus: false,
                    show: true,
                    is_movable: false,
                    is_resizable: false,
                    ..Default::default()
                },
                move |window, cx| {
                    match HasWindowHandle::window_handle(window).map(|h| h.as_raw()) {
                        Ok(RawWindowHandle::Win32(handle)) => {
                            platform::start(handle.hwnd.get(), overlay_shared);
                        }
                        _ => overlay_shared.status.store(3, Ordering::Relaxed),
                    }
                    cx.new(|_| Overlay)
                },
            );
            if let Err(error) = overlay {
                eprintln!("Could not create overlay: {error}");
                return;
            }
            let bounds = Bounds::centered(None, size(px(480.), px(280.)), cx);
            let result = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |_, cx| {
                    cx.new(|cx| {
                        cx.on_release(|_, cx| cx.quit()).detach();
                        cx.spawn(async move |entity, cx| {
                            loop {
                                Timer::after(Duration::from_millis(250)).await;
                                if entity.update(cx, |_, cx| cx.notify()).is_err() {
                                    break;
                                }
                            }
                        })
                        .detach();
                        Dashboard { shared }
                    })
                },
            );
            if let Err(error) = result {
                eprintln!("Could not create app window: {error}");
                cx.quit();
                return;
            }
            cx.activate(true);
        });
    }
}
#[cfg(windows)]
fn main() {
    windows_app::run();
}
#[cfg(not(windows))]
fn main() {
    eprintln!("League Companion requires Windows.");
}
