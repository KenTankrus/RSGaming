use auto_launch::AutoLaunchBuilder;

use crate::constants::APP_NAME;
use crate::errors::{AppError, AppResult};

// NOTE: the `auto-launch` crate's exact builder method names have not been
// checked against its docs in this session (no network access while
// writing this). If `set_use_launch_agent` doesn't exist on your installed
// version, drop that call -- it's a macOS-only knob and harmless to omit
// on Windows.
fn build_auto_launch() -> AppResult<auto_launch::AutoLaunch> {
    let exe_path = std::env::current_exe()?;
    AutoLaunchBuilder::new()
        .set_app_name(APP_NAME)
        .set_app_path(&exe_path.to_string_lossy())
        .set_use_launch_agent(true)
        .build()
        .map_err(|e| AppError::Other(format!("Failed to configure autostart: {e}")))
}

pub fn set_start_on_boot(enabled: bool) -> AppResult<()> {
    let auto = build_auto_launch()?;
    if enabled {
        auto.enable()
            .map_err(|e| AppError::Other(format!("Failed to enable autostart: {e}")))
    } else {
        auto.disable()
            .map_err(|e| AppError::Other(format!("Failed to disable autostart: {e}")))
    }
}

pub fn is_start_on_boot_enabled() -> bool {
    build_auto_launch()
        .and_then(|a| a.is_enabled().map_err(|e| AppError::Other(e.to_string())))
        .unwrap_or(false)
}
