use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};

use crate::constants::{
    APP_NAME, APP_ORG, APP_QUALIFIER, CACHE_FILE, CONFIG_FILE, INVESTMENTS_FILE, SCHEDULES_FILE,
};
use crate::errors::{AppError, AppResult};
use crate::models::portfolio::Portfolio;
use crate::models::schedule::ScheduleStore;
use crate::models::settings::AppSettings;

/// Returns (creating if necessary) the per-user application data directory,
/// e.g. `%APPDATA%\RSGEWatch\RSGEWatch\data` on Windows.
pub fn data_dir() -> AppResult<PathBuf> {
    let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME).ok_or(AppError::NoDataDir)?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn load_json<T: DeserializeOwned + Default>(path: &Path) -> AppResult<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn save_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, text)?;
    Ok(())
}

pub fn load_settings() -> AppResult<AppSettings> {
    load_json(&data_dir()?.join(CONFIG_FILE))
}

pub fn save_settings(settings: &AppSettings) -> AppResult<()> {
    save_json(&data_dir()?.join(CONFIG_FILE), settings)
}

pub fn load_portfolio() -> AppResult<Portfolio> {
    load_json(&data_dir()?.join(INVESTMENTS_FILE))
}

pub fn save_portfolio(portfolio: &Portfolio) -> AppResult<()> {
    save_json(&data_dir()?.join(INVESTMENTS_FILE), portfolio)
}

pub fn load_schedules() -> AppResult<ScheduleStore> {
    load_json(&data_dir()?.join(SCHEDULES_FILE))
}

pub fn save_schedules(store: &ScheduleStore) -> AppResult<()> {
    save_json(&data_dir()?.join(SCHEDULES_FILE), store)
}

// Reserved for a future caching need (e.g. caching known-item GE lookups,
// or last-good prices as a fallback if the API is unreachable at startup)
// -- matches the original spec's suggested cache.json file. Not wired to
// anything yet, hence the #[allow]: intentional, not dead code.
#[allow(dead_code)]
pub fn load_cache<T: DeserializeOwned + Default>() -> AppResult<T> {
    load_json(&data_dir()?.join(CACHE_FILE))
}

#[allow(dead_code)]
pub fn save_cache<T: Serialize>(value: &T) -> AppResult<()> {
    save_json(&data_dir()?.join(CACHE_FILE), value)
}
