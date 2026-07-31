// Hides the console window in release builds only, so `println!`/`tracing`
// output is still visible when running `cargo run` in development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod constants;
mod errors;
mod gui;
mod models;
mod notifications;
mod services;

use raw_window_handle::HasWindowHandle;

use app::application::RsgeWatchApp;
use app::state::AppState;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("{}", constants::APP_BANNER);
    tracing::info!("RSGEWatch v{}", constants::VERSION);

    // A multi-threaded Tokio runtime is kept alive for the whole program.
    // eframe blocks the main thread running the GUI event loop, so the GUI
    // uses `state.tokio_handle` to spawn one-off async work (Telegram test
    // sends, GE lookups), while the scheduler task below runs continuously
    // in the background for as long as the app is alive.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");
    let handle = runtime.handle().clone();

    let state = AppState::load(handle.clone());

    {
        let portfolio = state.portfolio.clone();
        let settings = state.settings.clone();
        let schedules = state.schedules.clone();
        handle.spawn(services::scheduler::run(portfolio, settings, schedules));
    }

    let start_minimized = state.settings.lock().unwrap().start_minimized;

    // NOTE: `IconData`'s field names and `with_icon`'s exact parameter type
    // (it may want `Arc<IconData>` on some egui/eframe versions rather than
    // a bare `IconData`) haven't been checked against a live build in this
    // session -- adjust if this doesn't compile as-is.
    let (icon_rgba, icon_w, icon_h) = app::icon::render_icon_rgba(4, app::icon::ICON_BG, app::icon::ICON_FG);

    let viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([960.0, 640.0])
        .with_icon(eframe::egui::IconData {
            rgba: icon_rgba,
            width: icon_w,
            height: icon_h,
        })
        .with_visible(!start_minimized);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        constants::APP_NAME,
        native_options,
        Box::new(move |cc| {
            // The tray has to be built here, inside this closure, because
            // this is the first point the real window handle (HWND) is
            // available -- see app::tray module docs for why the tray's
            // show/hide logic bypasses egui entirely and needs that raw
            // handle directly.
            let raw_window_handle::RawWindowHandle::Win32(handle) =
                cc.window_handle().expect("no window handle available").as_raw()
            else {
                panic!("RSGEWatch's tray/minimize-to-tray integration currently only supports Windows");
            };
            let hwnd = windows::Win32::Foundation::HWND(handle.hwnd.get() as _);
            let tray_icon = app::tray::install(hwnd);

            Ok(Box::new(RsgeWatchApp::new(state, tray_icon)) as Box<dyn eframe::App>)
        }),
    )
}
