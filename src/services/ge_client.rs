use std::collections::HashMap;

use crate::constants::USER_AGENT;
use crate::errors::{AppError, AppResult};
use crate::models::investment::Game;

/// Thin wrapper around the Grand Exchange price API.
///
/// Uses the Weird Gloop exchange-history API (`api.weirdgloop.org`) for
/// both RS3 and OSRS. An earlier version of this file tried switching RS3
/// to a newer-looking endpoint at `prices.runescape.wiki/api/v1/rs`,
/// reported by a third-party (unofficial) MCP server's README -- that
/// turned out to 404 in practice, meaning either the path was wrong or
/// that endpoint isn't actually live the way the README implied. Rather
/// than guess a second time, this reverts to the endpoint that was
/// structurally confirmed working (it was returning data before hitting a
/// 403, which turned out to be a missing `User-Agent` header, not a dead
/// endpoint -- see `constants::USER_AGENT`).
///
/// If you want to pursue the richer high/low real-time data later: the
/// most reliable way to find the *actual* endpoint prices.runescape.wiki
/// uses is to open its Network tab in a browser while viewing an item page
/// (e.g. https://prices.runescape.wiki/rs/item/29492) -- that page is a
/// client-side app that must be calling some real API to render its
/// high/low/quick-buy/quick-sell figures, and the Network tab will show
/// the exact URL and response shape it's actually hitting.
pub struct GeClient {
    http: reqwest::Client,
}

const GE_API_ROOT: &str = "https://api.weirdgloop.org/exchange/history";

impl GeClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }

    /// Fetches the latest known price for a batch of item IDs, all
    /// belonging to the same game (RS3 or OSRS -- their item ID spaces
    /// don't overlap, so a batch can't mix the two). Items that fail to
    /// resolve are simply omitted from the result map rather than failing
    /// the whole batch.
    pub async fn fetch_prices(&self, game: Game, item_ids: &[u32]) -> AppResult<HashMap<u32, u64>> {
        if item_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let ids_param = item_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let url = format!("{GE_API_ROOT}/{}/latest?id={ids_param}", game.api_key());

        let resp = self.http.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(AppError::GeApi(format!(
                "Unexpected status {} from Grand Exchange API",
                resp.status()
            )));
        }

        let raw: serde_json::Value = resp.json().await?;
        let mut out = HashMap::new();

        match raw {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if let Ok(id) = key.parse::<u32>() {
                        if let Some(price) = extract_price(&value) {
                            out.insert(id, price);
                        }
                    }
                }
            }
            other if item_ids.len() == 1 => {
                if let Some(price) = extract_price(&other) {
                    out.insert(item_ids[0], price);
                }
            }
            _ => {}
        }

        Ok(out)
    }
}

impl Default for GeClient {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_price(value: &serde_json::Value) -> Option<u64> {
    // The `latest` endpoint returns a flat object per item, but fall back
    // to unwrapping an array (as seen on the `all`/history endpoints) just
    // in case a `latest` response is ever shaped that way too.
    let point = value.as_array().and_then(|a| a.last()).unwrap_or(value);
    point.get("price").and_then(|p| p.as_u64())
}
