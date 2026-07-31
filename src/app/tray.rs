use std::sync::atomic::{AtomicIsize, Ordering};

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOWDEFAULT};

/// Why this module looks the way it does (read this before changing it):
///
/// eframe/egui's `update()` is only called while the window is visible and
/// "interactable". Once a window is hidden/minimized, egui simply stops
/// calling `update()` -- confirmed by the egui maintainers themselves
/// (emilk/egui discussions #737 and issue #3655): "egui doesn't update once
/// the window is not interactable e.g. invisible/minimized". An earlier
/// version of this app tried to route tray clicks through
/// `ctx.send_viewport_cmd(ViewportCommand::Visible(true))` from inside
/// `update()` -- but if the window is already hidden, `update()` isn't
/// running, so there's nothing left to process that command. That's the
/// actual reason double-click-to-restore (and by extension, anything else
/// routed through egui after minimizing) didn't work, separate from the
/// earlier message-pump bug this file also used to have.
///
/// The fix used here, matching the community's confirmed working solution:
/// register `tray-icon`'s event handlers as global callbacks
/// (`TrayIconEvent::set_event_handler` / `MenuEvent::set_event_handler`).
/// These fire directly from a raw Win32 message hook the moment Windows
/// delivers a shell notification -- independent of egui's frame loop
/// entirely -- and show/hide the window with a direct `ShowWindow` call via
/// the `windows` crate on the raw HWND, rather than going through egui at
/// all. "Exit" calls `std::process::exit(0)` directly for the same reason:
/// there's no guarantee anything is currently running to process a more
/// graceful shutdown command if the window is hidden. Investment/settings
/// data is saved to disk on every change (see `services::storage`), so
/// there's nothing that needs flushing on exit.
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

fn main_hwnd() -> Option<HWND> {
    let raw = MAIN_HWND.load(Ordering::SeqCst);
    if raw == 0 {
        None
    } else {
        Some(HWND(raw as _))
    }
}

pub fn show_main_window() {
    if let Some(hwnd) = main_hwnd() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
        }
    }
}

pub fn hide_main_window() {
    if let Some(hwnd) = main_hwnd() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
}

fn generate_icon() -> tray_icon::Icon {
    let (rgba, w, h) = crate::app::icon::render_icon_rgba(4, crate::app::icon::ICON_BG, crate::app::icon::ICON_FG);
    tray_icon::Icon::from_rgba(rgba, w, h).expect("failed to build tray icon")
}

/// Builds the tray icon and registers its event handlers. Must be called
/// once, from inside eframe's `AppCreator` closure (i.e. the `Box::new(|cc|
/// {...})` passed to `eframe::run_native`), using the HWND obtained from
/// `cc.window_handle()` -- see `main.rs`. The returned `TrayIcon` must be
/// kept alive for as long as the tray icon should exist (store it
/// somewhere it won't be dropped, e.g. as a field on the `App`).
///
/// NOTE: `TrayIconEvent`/`MenuEvent`'s exact shapes (`DoubleClick { button,
/// .. }`, `set_event_handler`, etc.) can shift slightly between
/// `tray-icon` versions and haven't been checked against a live build in
/// this session -- if this doesn't compile as-is, check
/// `tray_icon::TrayIconEvent`/`tray_icon::menu::MenuEvent`'s definitions
/// for your installed version and adjust.
pub fn install(hwnd: HWND) -> TrayIcon {
    MAIN_HWND.store(hwnd.0 as isize, Ordering::SeqCst);

    let menu = Menu::new();
    let open_item = MenuItem::new("Open RSGEWatch", true, None);
    let quit_item = MenuItem::new("Exit", true, None);
    let _ = menu.append(&open_item);
    let _ = menu.append(&quit_item);
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();

    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("RSGEWatch")
        .with_icon(generate_icon())
        .build()
        .expect("failed to create tray icon");

    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        if let TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } = event
        {
            show_main_window();
        }
    }));

    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_id {
            show_main_window();
        } else if event.id == quit_id {
            std::process::exit(0);
        }
    }));

    icon
}
