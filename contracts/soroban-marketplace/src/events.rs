// events.rs — Defines all contract event schemas for Afristore Marketplace

use soroban_sdk::{contractevent, contracttype, symbol_short, Address, Bytes, Symbol};

// Versioned event topics as Symbol constants
pub const LISTING_CREATED: Symbol = symbol_short!("lst_crtd");
pub const ARTWORK_SOLD: Symbol = symbol_short!("art_sold");
pub const LISTING_CANCELLED: Symbol = symbol_short!("lst_cncl");
pub const LISTING_UPDATED: Symbol = symbol_short!("lst_updt");
pub const BID_PLACED: Symbol = symbol_short!("bid_plcd");
pub const AUCTION_RESOLVED: Symbol = symbol_short!("auc_rslv");
pub const OFFER_MADE: Symbol = symbol_short!("ofr_made");
pub const OFFER_ACCEPTED: Symbol = symbol_short!("ofr_accp");
pub const OFFER_REJECTED: Symbol = symbol_short!("ofr_rjct");
pub const OFFER_WITHDRAWN: Symbol = symbol_short!("ofr_wdrn");
pub const ROYALTY_PAID: Symbol = symbol_short!("roy_paid");
pub const ARTIST_REVOKED: Symbol = symbol_short!("art_rvkd");
pub const ARTIST_REINSTATED: Symbol = symbol_short!("art_rnst");

// Event data structs
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingCreatedEvent {
    pub listing_id: u64,
    pub artist: Address,
    pub price: i128,
    pub currency: Symbol,
    pub metadata_cid: Bytes,
    pub ledger_sequence: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtworkSoldEvent {
    pub listing_id: u64,
    pub artist: Address,
    pub buyer: Address,
    pub price: i128,
    pub currency: Symbol,
    pub ledger_sequence: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingCancelledEvent {
    pub listing_id: u64,
    pub artist: Address,
    pub ledger_sequence: u32,
}

// Add more event structs as needed for other actions
