// ------------------------------------------------------------
// contract.rs — Afristore Marketplace contract implementation
// ------------------------------------------------------------

#[allow(unused_imports)]
use soroban_sdk::{
    contract, contractimpl, log, panic_with_error, token::Client as TokenClient, Address, Bytes,
    Env, Symbol, Vec,
};

use crate::{
    storage::{
        add_artist_listing_id, get_artist_listing_ids, get_listing_count, increment_listing_count,
        load_listing, save_listing,
    },
    types::{Listing, ListingStatus, MarketplaceError, Recipient},
};

// ────────────────────────────────────────────────────────────

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    // ── create_listing ───────────────────────────────────────
    /// Artist creates a new listing.
    ///
    /// * `metadata_cid` — raw bytes of the IPFS CID string
    /// * `price`        — price in stroops (i128, must be > 0)
    /// * `currency`     — must be `Symbol::short("XLM")` for MVP
    pub fn create_listing(
        env: Env,
        artist: Address,
        metadata_cid: Bytes,
        price: i128,
        currency: Symbol,
        recipients: Vec<Recipient>,
    ) -> u64 {
        // Require the artist to have authorised this call.
        artist.require_auth();

        // Validate inputs.
        if metadata_cid.is_empty() {
            panic_with_error!(&env, MarketplaceError::InvalidCid);
        }
        if price <= 0 {
            panic_with_error!(&env, MarketplaceError::InvalidPrice);
        }

        let recipients_len = recipients.len();
        if recipients_len == 0 || recipients_len > 4 {
            panic_with_error!(&env, MarketplaceError::TooManyRecipients);
        }

        let mut total_percentage = 0;
        for i in 0..recipients_len {
            let recipient = recipients.get(i).unwrap();
            total_percentage += recipient.percentage;
        }

        if total_percentage != 100 {
            panic_with_error!(&env, MarketplaceError::InvalidSplit);
        }

        let listing_id = increment_listing_count(&env);

        let listing = Listing {
            listing_id,
            artist: artist.clone(),
            metadata_cid,
            price,
            currency,
            recipients,
            status: ListingStatus::Active,
            owner: None,
            created_at: env.ledger().sequence(),
        };

        save_listing(&env, &listing);
        add_artist_listing_id(&env, &artist, listing_id);

        log!(
            &env,
            "Listing created: id={}, artist={}",
            listing_id,
            artist
        );

        listing_id
    }

    // ── buy_artwork ──────────────────────────────────────────
    /// Buyer purchases an active listing by paying the listed price in XLM.
    ///
    /// The contract:
    /// 1. Validates the listing is Active.
    /// 2. Transfers `price` stroops from `buyer` → contract.
    /// 3. Transfers `price` stroops from contract → `artist`.
    /// 4. Marks the listing Sold and records the buyer as owner.
    pub fn buy_artwork(env: Env, buyer: Address, listing_id: u64) -> bool {
        buyer.require_auth();

        let mut listing = load_listing(&env, listing_id)
            .unwrap_or_else(|| panic_with_error!(&env, MarketplaceError::ListingNotFound));

        if listing.status != ListingStatus::Active {
            panic_with_error!(&env, MarketplaceError::ListingNotActive);
        }
        if listing.artist == buyer {
            panic_with_error!(&env, MarketplaceError::CannotBuyOwnListing);
        }

        // Transfer payment: buyer → this contract → artist.
        // In unit tests, native token contract state is not available in the host
        // by default, so we skip transfer calls and validate state transitions.
        #[cfg(not(test))]
        {
            let token = TokenClient::new(&env, &Self::xlm_token_address(&env));
            token.transfer(&buyer, &env.current_contract_address(), &listing.price);

            let recipients_len = listing.recipients.len();
            let mut distributed_so_far: i128 = 0;

            for i in 0..recipients_len {
                let recipient = listing.recipients.get(i).unwrap();
                let amount_to_transfer = if i == recipients_len - 1 {
                    // Fractional rounding security: exact remainder to the final recipient directly.
                    listing.price - distributed_so_far
                } else {
                    (listing.price * recipient.percentage as i128) / 100
                };

                token.transfer(
                    &env.current_contract_address(),
                    &recipient.address,
                    &amount_to_transfer,
                );
                distributed_so_far += amount_to_transfer;
            }
        }

        // Update listing state.
        listing.status = ListingStatus::Sold;
        listing.owner = Some(buyer.clone());
        save_listing(&env, &listing);

        log!(
            &env,
            "Artwork sold: listing_id={}, buyer={}, price={}",
            listing_id,
            buyer,
            listing.price
        );

        true
    }

    // ── cancel_listing ───────────────────────────────────────
    /// Artist cancels their own active listing.
    pub fn cancel_listing(env: Env, artist: Address, listing_id: u64) -> bool {
        artist.require_auth();

        let mut listing = load_listing(&env, listing_id)
            .unwrap_or_else(|| panic_with_error!(&env, MarketplaceError::ListingNotFound));

        if listing.artist != artist {
            panic_with_error!(&env, MarketplaceError::Unauthorized);
        }
        if listing.status != ListingStatus::Active {
            panic_with_error!(&env, MarketplaceError::ListingNotActive);
        }

        listing.status = ListingStatus::Cancelled;
        save_listing(&env, &listing);

        log!(&env, "Listing cancelled: id={}", listing_id);
        true
    }

    // ── get_listing ──────────────────────────────────────────
    /// Returns the full Listing struct for a given ID.
    /// Panics with `ListingNotFound` if the ID does not exist.
    pub fn get_listing(env: Env, listing_id: u64) -> Listing {
        load_listing(&env, listing_id)
            .unwrap_or_else(|| panic_with_error!(&env, MarketplaceError::ListingNotFound))
    }

    // ── get_total_listings ───────────────────────────────────
    /// Returns the total number of listings ever created (counter, not active count).
    pub fn get_total_listings(env: Env) -> u64 {
        get_listing_count(&env)
    }

    // ── get_artist_listings ──────────────────────────────────
    /// Returns the Vec of listing IDs created by a given artist address.
    pub fn get_artist_listings(env: Env, artist: Address) -> Vec<u64> {
        get_artist_listing_ids(&env, &artist)
    }

    // ── Internal helpers ─────────────────────────────────────

    /// Returns the Stellar native asset (XLM) Soroban contract address.
    ///
    /// `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` is the
    /// well-known, deterministic contract ID for the native XLM asset on
    /// every Stellar network (both testnet and mainnet).
    #[cfg(not(test))]
    fn xlm_token_address(env: &Env) -> Address {
        Address::from_string_bytes(&soroban_sdk::Bytes::from_slice(
            env,
            b"CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
        ))
    }
}
