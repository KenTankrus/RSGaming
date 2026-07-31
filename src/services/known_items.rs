use crate::models::investment::Game;

pub struct KnownItem {
    pub name: &'static str,
    pub item_id: u32,
    pub game: Game,
}

/// A small, hand-curated shortlist used to resolve a typed item name (e.g.
/// "bond") to a Grand Exchange item ID.
///
/// Neither RS3 nor OSRS exposes a simple "search all items by free text"
/// endpoint: RS3's official catalogue (see `services::rs_item_lookup`) only
/// supports browsing by category + starting letter, which would take
/// dozens of requests to cover every category for a single search term.
/// Rather than build that (or guess at IDs I can't verify), this table
/// covers a couple of items confirmed against multiple independent sources
/// (RuneScape Wiki, the official secure.runescape.com item page, and
/// Grand Exchange Central) this session. Anything not listed here can
/// still be tracked by entering its numeric item ID directly -- find it on
/// the item's RuneScape Wiki / OSRS Wiki page (in the infobox) -- and the
/// "Look up on GE" button will confirm it and pull its current price.
///
/// Deliberately kept small and verified rather than large and guessed:
/// party hats, h'ween masks, etc. can be added here once their IDs are
/// confirmed, following the same pattern.
pub const KNOWN_ITEMS: &[KnownItem] = &[
    KnownItem {
        name: "Bond",
        item_id: 29492,
        game: Game::Rs3,
    },
    KnownItem {
        name: "Old school bond",
        item_id: 13190,
        game: Game::Osrs,
    },
];

/// Case-insensitive substring search over the known-items shortlist,
/// restricted to the given game.
pub fn search(query: &str, game: Game) -> Vec<&'static KnownItem> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    KNOWN_ITEMS
        .iter()
        .filter(|item| item.game == game && item.name.to_lowercase().contains(&q))
        .collect()
}
