pub mod telegram_sender;

// Re-exported as part of this module's public API for anything built on
// top of the NotificationSender trait -- not yet used internally, since
// services::scheduler currently calls services::telegram directly rather
// than going through the trait object. Hence the #[allow]: it's intentional,
// not dead code.
#[allow(unused_imports)]
pub use telegram_sender::TelegramSender;

// Future expansion (see spec's "Future Expansion" section): add
// discord_sender.rs / email_sender.rs / desktop_sender.rs here, each
// implementing `crate::models::notification::NotificationSender`, then
// register them alongside `TelegramSender` wherever notifications are
// dispatched (currently `services::scheduler::run`). No changes to
// scheduling or calculation logic should be needed.
