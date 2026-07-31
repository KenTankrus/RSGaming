use async_trait::async_trait;

use crate::errors::AppResult;

/// Common interface for notification channels. Telegram is the only real
/// implementation today (see `notifications::TelegramSender`); Discord,
/// email, and desktop-toast notifications (see project roadmap) can be
/// added later by implementing this trait and registering them alongside
/// Telegram in the scheduler, without touching scheduling/calculation logic.
///
/// Not yet used internally -- services::scheduler currently calls
/// services::telegram directly rather than going through this trait, since
/// there's only one channel to dispatch to so far. Hence the #[allow]:
/// this is deliberate scaffolding, not dead code.
#[allow(dead_code)]
#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send(&self, message: &str) -> AppResult<()>;
    fn name(&self) -> &'static str;
}
