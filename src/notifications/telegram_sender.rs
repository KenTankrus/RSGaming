use async_trait::async_trait;

use crate::errors::AppResult;
use crate::models::notification::NotificationSender;
use crate::models::settings::TelegramConfig;
use crate::services::telegram;

/// Not yet constructed anywhere -- see NotificationSender's doc comment
/// for why. Deliberate scaffolding, not dead code.
#[allow(dead_code)]
pub struct TelegramSender {
    pub config: TelegramConfig,
}

#[async_trait]
impl NotificationSender for TelegramSender {
    async fn send(&self, message: &str) -> AppResult<()> {
        telegram::send_message(&self.config, message).await
    }

    fn name(&self) -> &'static str {
        "Telegram"
    }
}
