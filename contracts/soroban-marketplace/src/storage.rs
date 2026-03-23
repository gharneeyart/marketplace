// ------------------------------------------------------------
// storage.rs — Ledger storage key helpers
// ------------------------------------------------------------
//
// Key design
// ──────────
// Persistent storage:
//   DataKey::Listing(u64)          → Listing struct
//   DataKey::ArtistListings(Address) → Vec<u64>  (artist's listing IDs)
//   DataKey::ListingCount          → u64  (auto-increment counter)
//
// All values use `env.storage().persistent()` so they survive
// ledger archival (TTL is extended on every write).
// ------------------------------------------------------------

use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::Listing;

/// Storage key variants for the marketplace contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Stores the global listing counter (u64).
    ListingCount,
    /// Stores a single `Listing` by its ID.
    Listing(u64),
    /// Stores a `Vec<u64>` of listing IDs owned by an artist.
    ArtistListings(Address),
}

// ── Bump amounts (ledger sequences) ─────────────────────────
/// Keep persistent entries alive for ~30 days at 6s/ledger.
const LEDGER_TTL_BUMP: u32 = 432_000;
/// Threshold before we extend: re-bump when closer than 10 days out.
const LEDGER_TTL_THRESHOLD: u32 = 144_000;

// ── Counter helpers ──────────────────────────────────────────

pub fn get_listing_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get::<DataKey, u64>(&DataKey::ListingCount)
        .unwrap_or(0)
}

pub fn increment_listing_count(env: &Env) -> u64 {
    let count = get_listing_count(env) + 1;
    env.storage()
        .persistent()
        .set(&DataKey::ListingCount, &count);
    env.storage().persistent().extend_ttl(
        &DataKey::ListingCount,
        LEDGER_TTL_THRESHOLD,
        LEDGER_TTL_BUMP,
    );
    count
}

// ── Listing CRUD ─────────────────────────────────────────────

pub fn save_listing(env: &Env, listing: &Listing) {
    let key = DataKey::Listing(listing.listing_id);
    env.storage().persistent().set(&key, listing);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_TTL_THRESHOLD, LEDGER_TTL_BUMP);
}

pub fn load_listing(env: &Env, listing_id: u64) -> Option<Listing> {
    let key = DataKey::Listing(listing_id);
    let result = env.storage().persistent().get::<DataKey, Listing>(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_TTL_THRESHOLD, LEDGER_TTL_BUMP);
    }
    result
}

// ── Artist listing index ─────────────────────────────────────

pub fn get_artist_listing_ids(env: &Env, artist: &Address) -> Vec<u64> {
    let key = DataKey::ArtistListings(artist.clone());
    env.storage()
        .persistent()
        .get::<DataKey, Vec<u64>>(&key)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_artist_listing_id(env: &Env, artist: &Address, listing_id: u64) {
    let key = DataKey::ArtistListings(artist.clone());
    let mut ids = get_artist_listing_ids(env, artist);
    ids.push_back(listing_id);
    env.storage().persistent().set(&key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_TTL_THRESHOLD, LEDGER_TTL_BUMP);
}
