use eframe::egui;

use crate::app::state::AppState;
use crate::services::{storage, telegram};

pub fn show(ui: &mut egui::Ui, ctx: &egui::Context, state: &AppState, ui_state: &mut crate::gui::UiState) {
    ui.heading("Telegram Notifications");
    ui.add_space(8.0);

    let mut settings = state.settings.lock().unwrap();
    let mut changed = false;

    changed |= ui
        .checkbox(&mut settings.telegram.enabled, "Enable Telegram notifications")
        .changed();

    ui.label("Bot token");
    changed |= ui
        .add(
            egui::TextEdit::singleline(&mut settings.telegram.bot_token)
                .password(!ui_state.telegram_show_secrets),
        )
        .changed();

    ui.label("Chat ID");
    changed |= ui
        .add(
            egui::TextEdit::singleline(&mut settings.telegram.chat_id)
                .password(!ui_state.telegram_show_secrets),
        )
        .changed();

    ui.horizontal(|ui| {
        if ui
            .small_button(if ui_state.telegram_show_secrets { "Hide" } else { "Show" })
            .clicked()
        {
            ui_state.telegram_show_secrets = !ui_state.telegram_show_secrets;
        }
        ui.label("Show or hide Telegram token and chat ID.");
    });

    ui.add_space(4.0);
    ui.hyperlink_to(
        "How can I get Telegram bot token and chat ID? - Tracardi Documentation",
        "https://docs.tracardi.com/qa/how_can_i_get_telegram_bot/",
    );

    if changed {
        if let Err(e) = storage::save_settings(&settings) {
            tracing::warn!("Failed to save settings: {e}");
        }
    }

    ui.add_space(8.0);

    if ui.button("Send test message").clicked() {
        let cfg = settings.telegram.clone();
        let ctx = ctx.clone();
        let status = state.telegram_status.clone();
        *status.lock().unwrap() = Some("Sending...".to_string());

        state.tokio_handle.spawn(async move {
            let msg = match telegram::test_connection(&cfg).await {
                Ok(()) => "Test message sent successfully.".to_string(),
                Err(e) => format!("Failed: {e}"),
            };
            *status.lock().unwrap() = Some(msg);
            ctx.request_repaint();
        });
    }

    if let Some(status) = state.telegram_status.lock().unwrap().as_ref() {
        ui.label(status);
    }
}
