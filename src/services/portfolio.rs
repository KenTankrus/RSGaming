use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::errors::AppResult;
use crate::models::investment::{Game, InvestmentStatus};
use crate::models::portfolio::Portfolio;
use crate::services::ge_client::GeClient;

#[derive(Debug, Clone, Copy, Default)]
pub struct RefreshSummary {
    pub updated: usize,
    /// Active investments with no item ID set -- these can never be priced
    /// automatically. Surfaced so it's obvious *why* an investment isn't
    /// showing a live price/gain, rather than that just silently sitting
    /// there looking broken.
    pub missing_item_id: usize,
    /// Active investments that had an item ID but the API didn't return a
    /// price for (wrong/delisted ID, or a transient API issue).
    pub unresolved: usize,
}

/// Refreshes current prices for every active investment that has a known
/// item ID. Investments are grouped by `Game` first, since RS3 and OSRS
/// item IDs live in separate spaces and are fetched via separate API
/// calls. The portfolio lock is only held for the synchronous bookkeeping
/// before and after the network calls -- never across an `.await` --  so
/// this is safe to call from a task spawned on a multi-threaded Tokio
/// runtime.
pub async fn refresh_prices(
    portfolio: &Arc<Mutex<Portfolio>>,
    client: &GeClient,
) -> AppResult<RefreshSummary> {
    let mut missing_item_id = 0usize;
    let ids_by_game: HashMap<Game, Vec<u32>> = {
        let pf = portfolio.lock().unwrap();
        let mut map: HashMap<Game, Vec<u32>> = HashMap::new();
        for inv in pf
            .investments
            .iter()
            .filter(|i| i.status == InvestmentStatus::Active)
        {
            match inv.item_id {
                Some(id) => map.entry(inv.game).or_default().push(id),
                None => missing_item_id += 1,
            }
        }
        map
    };

    if ids_by_game.is_empty() {
        return Ok(RefreshSummary {
            updated: 0,
            missing_item_id,
            unresolved: 0,
        });
    }

    let mut prices: HashMap<(Game, u32), u64> = HashMap::new();
    for (game, ids) in &ids_by_game {
        let fetched = client.fetch_prices(*game, ids).await?;
        for (id, price) in fetched {
            prices.insert((*game, id), price);
        }
    }

    let now = Utc::now();
    let mut updated = 0usize;
    let mut unresolved = 0usize;
    {
        let mut pf = portfolio.lock().unwrap();
        for inv in pf.investments.iter_mut() {
            if inv.status != InvestmentStatus::Active {
                continue;
            }
            if let Some(id) = inv.item_id {
                match prices.get(&(inv.game, id)) {
                    Some(&price) => {
                        inv.previous_price = inv.current_price;
                        inv.current_price = Some(price);
                        inv.last_updated = Some(now);
                        updated += 1;
                    }
                    None => unresolved += 1,
                }
            }
        }
    }

    Ok(RefreshSummary {
        updated,
        missing_item_id,
        unresolved,
    })
}
