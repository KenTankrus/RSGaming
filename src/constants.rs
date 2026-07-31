pub const APP_NAME: &str = "RSGEWatch";
pub const APP_ORG: &str = "RSGEWatch";
pub const APP_QUALIFIER: &str = "dev";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const APP_BANNER: &str = r#"
================================
       Welcome to RSGEWatch
================================
"#;

// Storage file names, all stored as JSON under the OS app-data directory
// (see services::storage::data_dir).
pub const CONFIG_FILE: &str = "config.json";
pub const INVESTMENTS_FILE: &str = "investments.json";
pub const SCHEDULES_FILE: &str = "schedules.json";
pub const CACHE_FILE: &str = "cache.json";

/// The RuneScape/OSRS Wiki's API usage policy explicitly requires a
/// descriptive User-Agent identifying the tool making requests (and asks,
/// if you're willing, for contact info) -- requests without one are
/// pre-emptively blocked. Confirmed via runescape.wiki/w/Help:APIs.
/// Personalize the contact bit if you'd like (a GitHub URL, email, or
/// Discord handle all work).
pub const USER_AGENT: &str = concat!(
    "RSGEWatch/",
    env!("CARGO_PKG_VERSION"),
    " (personal RS3/OSRS Grand Exchange investment tracker)"
);

// Grand Exchange API base URLs live in services::ge_client (there are two
// now: the new prices.runescape.wiki real-time API for RS3, and the older
// api.weirdgloop.org one still used for OSRS), rather than as constants
// here, since each needs the game segment inserted into its path.

/// How often the background scheduler wakes up to check whether it's time
/// to refresh prices or evaluate schedules/event rules.
pub const SCHEDULER_TICK_SECONDS: u64 = 60;

pub const DEFAULT_REFRESH_MINUTES: u64 = 60;
