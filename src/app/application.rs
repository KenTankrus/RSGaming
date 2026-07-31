use std::time::Duration;

use eframe::egui;
use tray_icon::TrayIcon;

use crate::app::state::AppState;
use crate::gui;

pub enum CloseAction {
    Cancel,
    Minimize,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Investments,
    Schedules,
    Telegram,
    Statistics,
    Settings,
}

pub struct RsgeWatchApp {
    pub state: AppState,
    pub tab: Tab,
    /// Kept alive only so the tray icon isn't dropped (and thus removed)
    /// -- all of its actual event handling happens outside egui entirely,
    /// via global callbacks registered in `app::tray::install`.
    _tray_icon: TrayIcon,

    show_close_dialog: bool,
    dont_ask_again: bool,

    pub ui_state: gui::UiState,
}

impl RsgeWatchApp {
    pub fn new(state: AppState, tray_icon: TrayIcon) -> Self {
        Self {
            state,
            tab: Tab::Dashboard,
            _tray_icon: tray_icon,

            show_close_dialog: false,
            dont_ask_again: false,

            ui_state: gui::UiState::default(),
        }
    }
}

impl eframe::App for RsgeWatchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Note: there is no tray-poll here anymore. Tray clicks (double
        // click to restore, Open/Exit menu items) are handled entirely
        // outside of egui's update loop -- see app::tray module docs for
        // why (short version: egui stops calling update() once the window
        // is hidden, so anything depending on update() to process a tray
        // click would never run while minimized).

        // Intercept the window's close button: hide to tray instead of
        // exiting. This only needs to work while the window is visible
        // (which it is, whenever this close-requested check fires), so
        // egui's viewport API is fine here -- the earlier hidden-window
        // limitation doesn't apply to this specific path.
        //
        // NOTE: `i.viewport().close_requested()` and
        // `ViewportCommand::CancelClose` rely on egui's multi-viewport API
        // (0.27+). Verify these names/methods still match your installed
        // egui/eframe version -- not checked live in this session.
        let close_requested: bool = ctx.input(|i| i.viewport().close_requested());

        if close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

            if self.dont_ask_again {
                crate::app::tray::hide_main_window();
            } else {
                self.show_close_dialog = true;
            }
        }

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.tab, Tab::Investments, "Investments");
                ui.selectable_value(&mut self.tab, Tab::Schedules, "Notifications");
                ui.selectable_value(&mut self.tab, Tab::Telegram, "Telegram");
                ui.selectable_value(&mut self.tab, Tab::Statistics, "Statistics");
                ui.selectable_value(&mut self.tab, Tab::Settings, "Settings");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Dashboard => gui::dashboard::show(ui, &self.state),
            Tab::Investments => gui::investments::show(ui, &self.state, &mut self.ui_state),
            Tab::Schedules => gui::schedules::show(ui, &self.state, &mut self.ui_state),
            Tab::Telegram => gui::telegram::show(ui, ctx, &self.state, &mut self.ui_state),
            Tab::Statistics => gui::statistics::show(ui, &self.state),
            Tab::Settings => gui::settings::show(ui, &self.state),
        });

        if self.show_close_dialog {
    egui::Window::new("Minimize RSGEWatch?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Keep RSGEWatch running in the background");

            ui.separator();

            ui.label(
                "RSGEWatch can continue monitoring your RuneScape investments while minimized to the system tray."
            );

            ui.add_space(10.0);

            ui.label("If you minimize:");

            ui.label("✓ Continue checking Grand Exchange prices");
            ui.label("✓ Continue monitoring your investments");
            ui.label("✓ Continue sending notifications");

            ui.add_space(10.0);

            ui.label("If you exit:");

            ui.label("• Monitoring stops");
            ui.label("• Price updates stop");
            ui.label("• Notifications stop until RSGEWatch is started again");

            ui.add_space(10.0);

            ui.checkbox(
                &mut self.dont_ask_again,
                "Don't ask me again",
            );

            ui.separator();

            let mut action = None;
            ui.horizontal(|ui| {
                if ui.button("Minimize to Tray").clicked() {
                    action = Some(CloseAction::Minimize);
                }
                if ui.button("Exit").clicked() {
                    action = Some(CloseAction::Exit);
                }
                if ui.button("Cancel").clicked() {
                    action = Some(CloseAction::Cancel);
                }
            });

            if let Some(action) = action {
                self.show_close_dialog = false;
                match action {
                    CloseAction::Minimize => {
                        crate::app::tray::hide_main_window();
                    }
                    CloseAction::Exit => {
                        // Data is saved to disk on every change, so an
                        // immediate exit here is safe -- see app::tray docs.
                        std::process::exit(0);
                    }
                    CloseAction::Cancel => {}
                }
            }
        });
}

        // Background price refresh / schedule checks happen off-thread;
        // keep repainting periodically so their results show up promptly.
        ctx.request_repaint_after(Duration::from_secs(2));
    }
}
