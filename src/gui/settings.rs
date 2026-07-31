use eframe::egui;

use crate::app::startup;
use crate::app::state::AppState;
use crate::services::storage;

pub fn show(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Settings");
    ui.add_space(8.0);

    let mut settings = state.settings.lock().unwrap();
    let mut changed = false;

    if ui
        .checkbox(&mut settings.start_on_boot, "Start automatically with Windows")
        .changed()
    {
        changed = true;
        if let Err(e) = startup::set_start_on_boot(settings.start_on_boot) {
            tracing::warn!("Failed to update autostart setting: {e}");
        }
    }

    if ui
        .checkbox(&mut settings.start_minimized, "Start minimized to tray")
        .changed()
    {
        changed = true;
    }

    ui.add_space(8.0);
    ui.label("Price refresh interval (minutes)");
    let mut interval_str = settings.refresh_interval_minutes.to_string();
    if ui.text_edit_singleline(&mut interval_str).changed() {
        if let Ok(v) = interval_str.trim().parse::<u64>() {
            settings.refresh_interval_minutes = v.max(1);
            changed = true;
        }
    }

    if changed {
        if let Err(e) = storage::save_settings(&settings) {
            tracing::warn!("Failed to save settings: {e}");
        }
    }

    ui.add_space(24.0);
    ui.separator();
    ui.weak(format!("RSGEWatch v{}", crate::constants::VERSION));
}
