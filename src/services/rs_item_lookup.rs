use crate::errors::{AppError, AppResult};
use crate::models::investment::Game;

#[derive(Debug, Clone)]
pub struct ItemDetail {
    pub id: u32,
    pub name: String,
    pub current_price: Option<u64>,
    pub members: bool,
}

/// Looks up a single RS3 item by ID against Jagex's official item
/// catalogue API. Confirmed this session against the RuneScape Wiki's
/// "Application programming interface" documentation page, including a
/// live example response shape:
/// `{"total":..,"items":[{"id":..,"name":"...","current":{"trend":"...",
/// "price":873}, "today":{...}, "members":"true", ...}]}` (from the
/// `catalogue/items.json` browse endpoint -- `catalogue/detail.json` for a
/// single item wraps the same per-item shape under a top-level `"item"`
/// key instead of a `"items"` array).
///
/// This is distinct from `GeClient::fetch_prices` (Weird Gloop API, used
/// for periodic batch price refreshes of everything in the portfolio) --
/// this one is for one-off "does this ID exist, what's it called, what's
/// it worth right now" lookups, e.g. when adding a new investment.
///
/// RS3 only: Jagex's equivalent OSRS item catalogue endpoint has a
/// checkered history of availability, and wasn't re-verified this session,
/// so OSRS investments currently rely on manually-entered item IDs plus
/// the periodic Weird Gloop price refresh rather than this lookup.
///
/// Price format note: the confirmed example above returned `"price":873`
/// as a plain JSON number, but Jagex's catalogue API has, at various
/// points, formatted larger prices as comma-separated or `k`/`m`/`b`-
/// suffixed strings (e.g. `"2.5m"`) instead. `parse_price_value` handles
/// both forms defensively so a format difference for expensive items
/// (bonds, party hats, etc.) doesn't silently produce a wrong number.
pub async fn search_rs3_items(query: &str) -> AppResult<Vec<ItemDetail>> {
    search_items(Game::Rs3, query).await
}

pub async fn search_osrs_items(query: &str) -> AppResult<Vec<ItemDetail>> {
    search_items(Game::Osrs, query).await
}

async fn search_items(game: Game, query: &str) -> AppResult<Vec<ItemDetail>> {
    let url = reqwest::Url::parse_with_params(
        &format!(
            "https://secure.runescape.com/{}/api/catalogue/search.json",
            item_db_prefix(game)
        ),
        [("query", query)],
    )
    .map_err(|e| AppError::Other(format!("Failed to build search URL: {e}")))?;

    let http = reqwest::Client::builder()
        .user_agent(crate::constants::USER_AGENT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = http.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::GeApi(format!(
            "Unexpected status {} from the item search API",
            resp.status()
        )));
    }

    let raw: serde_json::Value = resp.json().await?;
    let results = raw
        .get("results")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::GeApi("Item search response did not contain results".into()))?;

    let items = results.iter().filter_map(parse_search_result).collect();
    Ok(items)
}

fn parse_search_result(value: &serde_json::Value) -> Option<ItemDetail> {
    let id = value
        .get("id")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)?;
    let name = value.get("name")?.as_str()?.to_string();
    let current_price = value
        .get("current")
        .and_then(|c| c.get("price"))
        .and_then(parse_price_value);
    let members = value
        .get("members")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Some(ItemDetail {
        id,
        name,
        current_price,
        members,
    })
}

pub async fn lookup_rs3_item(item_id: u32) -> AppResult<ItemDetail> {
    lookup_item(Game::Rs3, item_id).await
}

pub async fn lookup_osrs_item(item_id: u32) -> AppResult<ItemDetail> {
    lookup_item(Game::Osrs, item_id).await
}

async fn lookup_item(game: Game, item_id: u32) -> AppResult<ItemDetail> {
    let url = format!(
        "https://secure.runescape.com/{}/api/catalogue/detail.json?item={item_id}",
        item_db_prefix(game)
    );

    let http = reqwest::Client::builder()
        .user_agent(crate::constants::USER_AGENT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = http.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::GeApi(format!(
            "Unexpected status {} from the RuneScape item catalogue",
            resp.status()
        )));
    }

    let raw: serde_json::Value = resp.json().await?;
    let item = raw.get("item").ok_or_else(|| {
        AppError::GeApi(format!(
            "Item {item_id} was not found in the Grand Exchange catalogue"
        ))
    })?;

    let name = item
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::GeApi("Item catalogue response was missing a name".into()))?
        .to_string();

    let current_price = item
        .get("current")
        .and_then(|c| c.get("price"))
        .and_then(parse_price_value);

    let members = item
        .get("members")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Ok(ItemDetail {
        id: item_id,
        name,
        current_price,
        members,
    })
}

fn item_db_prefix(game: Game) -> &'static str {
    match game {
        Game::Rs3 => "m=itemdb_rs",
        Game::Osrs => "m=itemdb_oldschool",
    }
}

fn parse_price_value(value: &serde_json::Value) -> Option<u64> {
    match value {
        serde_json::Value::Number(n) => n.as_u64().or_else(|| n.as_f64().map(|f| f.round() as u64)),
        serde_json::Value::String(s) => parse_price_string(s),
        _ => None,
    }
}

fn parse_price_string(s: &str) -> Option<u64> {
    let cleaned = s.trim().replace(',', "");
    if cleaned.is_empty() {
        return None;
    }
    let (num_part, multiplier) = match cleaned.chars().last() {
        Some(c) if c.eq_ignore_ascii_case(&'k') => (&cleaned[..cleaned.len() - 1], 1_000.0),
        Some(c) if c.eq_ignore_ascii_case(&'m') => (&cleaned[..cleaned.len() - 1], 1_000_000.0),
        Some(c) if c.eq_ignore_ascii_case(&'b') => (&cleaned[..cleaned.len() - 1], 1_000_000_000.0),
        _ => (cleaned.as_str(), 1.0),
    };
    num_part.trim().parse::<f64>().ok().map(|n| (n * multiplier).round() as u64)
}
