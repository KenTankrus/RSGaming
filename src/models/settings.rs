use serde::{Deserialize, Serialize};

use crate::constants::DEFAULT_REFRESH_MINUTES;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub start_on_boot: bool,
    pub start_minimized: bool,
    pub refresh_interval_minutes: u64,
    pub telegram: TelegramConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_on_boot: false,
            start_minimized: false,
            refresh_interval_minutes: DEFAULT_REFRESH_MINUTES,
            telegram: TelegramConfig::default(),
        }
    }
}
