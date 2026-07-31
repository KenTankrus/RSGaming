use serde::Serialize;

use crate::errors::{AppError, AppResult};
use crate::models::settings::TelegramConfig;

#[derive(Serialize)]
struct SendMessageRequest<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

/// Sends a message via the Telegram Bot API. A no-op (Ok) if Telegram
/// notifications are disabled in settings.
pub async fn send_message(config: &TelegramConfig, message: &str) -> AppResult<()> {
    if !config.enabled {
        return Ok(());
    }
    if config.bot_token.trim().is_empty() || config.chat_id.trim().is_empty() {
        return Err(AppError::Telegram(
            "Bot token or chat ID is not configured".into(),
        ));
    }

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );
    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .json(&SendMessageRequest {
            chat_id: &config.chat_id,
            text: message,
            parse_mode: "HTML",
        })
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Telegram(format!(
            "Telegram API responded with an error: {body}"
        )));
    }

    Ok(())
}

pub async fn test_connection(config: &TelegramConfig) -> AppResult<()> {
    send_message(
        config,
        "RSGEWatch is connected and ready to send alerts.",
    )
    .await
}
