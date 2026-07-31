use std::sync::{Arc, Mutex};

use crate::models::portfolio::Portfolio;
use crate::models::schedule::ScheduleStore;
use crate::models::settings::AppSettings;

pub type SharedPortfolio = Arc<Mutex<Portfolio>>;
pub type SharedSettings = Arc<Mutex<AppSettings>>;
pub type SharedSchedules = Arc<Mutex<ScheduleStore>>;

/// Outcome of a "Look up on GE" click in the Investments tab -- populated
/// by a spawned async task, read back by the GUI on the next frame.
pub enum GeLookupOutcome {
    Single(ResolvedItem),
    Multiple(Vec<ResolvedItem>),
}

pub struct GeLookupResult {
    /// What was being looked up (item ID or name), so the GUI can tell
    /// whether this result is still relevant to the current form state.
    pub query: String,
    pub outcome: Result<GeLookupOutcome, String>,
}

pub struct ResolvedItem {
    pub id: u32,
    pub name: String,
    pub current_price: Option<u64>,
    pub members: bool,
}

/// Cross-thread application state. Clones of the Arcs inside are held by
/// both the GUI (main thread, via eframe) and the background Tokio
/// scheduler task, so all mutation goes through `std::sync::Mutex`. Locks
/// are always released before any `.await` point (see
/// `services::scheduler` and `services::portfolio`) so this is safe even
/// though the guards aren't `Send`.
#[derive(Clone)]
pub struct AppState {
    pub portfolio: SharedPortfolio,
    pub settings: SharedSettings,
    pub schedules: SharedSchedules,
    /// Result of the last "Send test message" click, shown in the Telegram
    /// tab. Shared so the spawned async task can write the result back.
    pub telegram_status: Arc<Mutex<Option<String>>>,
    /// Result of the last "Look up on GE" click, shown in the Investments
    /// tab's add-investment form.
    pub ge_lookup: Arc<Mutex<Option<GeLookupResult>>>,
    /// Result + timestamp of the last manual "Refresh prices now" click,
    /// shown on the Dashboard.
    pub refresh_status: Arc<Mutex<Option<String>>>,
    pub tokio_handle: tokio::runtime::Handle,
}

impl AppState {
    pub fn load(tokio_handle: tokio::runtime::Handle) -> Self {
        let portfolio = crate::services::storage::load_portfolio().unwrap_or_default();
        let mut settings = crate::services::storage::load_settings().unwrap_or_default();
        let schedules = crate::services::storage::load_schedules().unwrap_or_default();

        // The saved config's `start_on_boot` flag can drift from reality
        // (e.g. the user removes the startup entry via Task Manager, or
        // Windows drops it). Reconcile against the actual registered state
        // on load so the Settings checkbox reflects what's really
        // happening, rather than trusting our own possibly-stale copy.
        let actually_enabled = crate::app::startup::is_start_on_boot_enabled();
        if settings.start_on_boot != actually_enabled {
            settings.start_on_boot = actually_enabled;
            if let Err(e) = crate::services::storage::save_settings(&settings) {
                tracing::warn!("Failed to persist reconciled start_on_boot setting: {e}");
            }
        }

        Self {
            portfolio: Arc::new(Mutex::new(portfolio)),
            settings: Arc::new(Mutex::new(settings)),
            schedules: Arc::new(Mutex::new(schedules)),
            telegram_status: Arc::new(Mutex::new(None)),
            ge_lookup: Arc::new(Mutex::new(None)),
            refresh_status: Arc::new(Mutex::new(None)),
            tokio_handle,
        }
    }
}
