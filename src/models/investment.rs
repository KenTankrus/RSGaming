use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentStatus {
    Active,
    Sold,
}

/// Which game an investment's item ID and price belong to. RS3 and OSRS
/// have entirely separate item ID spaces and Grand Exchange price data, so
/// this determines which Weird Gloop API route is used to price the item
/// (see `services::ge_client::GeClient::fetch_prices`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Game {
    Rs3,
    Osrs,
}

impl Game {
    /// The path segment Weird Gloop's exchange API uses for this game,
    /// e.g. https://api.weirdgloop.org/exchange/history/{key}/latest
    pub fn api_key(self) -> &'static str {
        match self {
            Game::Rs3 => "rs",
            Game::Osrs => "osrs",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Game::Rs3 => "RS3",
            Game::Osrs => "OSRS",
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::Rs3
    }
}

/// A single tracked Grand Exchange investment (a bond, a party hat, a
/// h'ween mask, etc). Raw, persisted fields only -- derived numbers like
/// profit/loss live in `services::calculations`, not here, so this struct
/// stays a plain data record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investment {
    pub id: String,
    pub item_name: String,
    pub item_id: Option<u32>,
    /// Defaults to RS3 when missing so investment files saved before this
    /// field existed still load correctly.
    #[serde(default)]
    pub game: Game,

    pub purchase_date: DateTime<Utc>,
    pub purchase_price: u64,
    pub quantity: u64,
    pub notes: Option<String>,

    pub status: InvestmentStatus,

    pub current_price: Option<u64>,
    pub previous_price: Option<u64>,
    pub last_updated: Option<DateTime<Utc>>,

    pub sold_price: Option<u64>,
    pub sold_date: Option<DateTime<Utc>>,
}
