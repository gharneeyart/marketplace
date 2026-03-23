// ------------------------------------------------------------
// test.rs — Unit tests for the Soroban marketplace contract
// ------------------------------------------------------------

use super::*;
use crate::types::Recipient;
use soroban_sdk::{bytes, symbol_short, testutils::Address as _, vec, Address, Env};

/// Helper — deploy the contract and return (env, client, artist, buyer, token_admin, token_id).
fn setup() -> (
    Env,
    MarketplaceContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    // ✅ use register() instead of register_contract()
    let contract_id = env.register(MarketplaceContract, ());
    let client = MarketplaceContractClient::new(&env, &contract_id);

    let artist = Address::generate(&env);
    let buyer = Address::generate(&env);

    (env, client, artist, buyer, contract_id)
}

fn valid_recipients(env: &Env, artist: &Address) -> soroban_sdk::Vec<Recipient> {
    vec![
        env,
        Recipient {
            address: artist.clone(),
            percentage: 100,
        },
    ]
}

// ── create_listing ───────────────────────────────────────────

#[test]
fn test_create_listing_success() {
    let (env, client, artist, _buyer, _) = setup();

    let cid = bytes!(&env, 0x516d546573744349444f6e495046533132333435);
    let price: i128 = 10_000_000; // 1 XLM

    let listing_id = client.create_listing(
        &artist,
        &cid,
        &price,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );

    assert_eq!(listing_id, 1);
    assert_eq!(client.get_total_listings(), 1);

    let listing = client.get_listing(&1);
    assert_eq!(listing.listing_id, 1);
    assert_eq!(listing.artist, artist);
    assert_eq!(listing.price, price);
    assert_eq!(listing.status, ListingStatus::Active);
    assert!(listing.owner.is_none());
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_create_listing_zero_price() {
    let (env, client, artist, _, _) = setup();
    let cid = bytes!(&env, 0x516d74657374);
    client.create_listing(
        &artist,
        &cid,
        &0_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_create_listing_empty_cid() {
    let (env, client, artist, _, _) = setup();
    client.create_listing(
        &artist,
        &bytes!(&env,),
        &10_000_000_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_create_listing_invalid_split() {
    let (env, client, artist, _, _) = setup();
    let cid = bytes!(&env, 0x516d74657374);
    let recipients = vec![
        &env,
        Recipient {
            address: artist.clone(),
            percentage: 50, // Doesn't equal 100
        },
    ];
    client.create_listing(
        &artist,
        &cid,
        &1_000_000_i128,
        &symbol_short!("XLM"),
        &recipients,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_create_listing_too_many_recipients() {
    let (env, client, artist, _, _) = setup();
    let cid = bytes!(&env, 0x516d74657374);
    let recipients = vec![
        &env,
        Recipient {
            address: Address::generate(&env),
            percentage: 20,
        },
        Recipient {
            address: Address::generate(&env),
            percentage: 20,
        },
        Recipient {
            address: Address::generate(&env),
            percentage: 20,
        },
        Recipient {
            address: Address::generate(&env),
            percentage: 20,
        },
        Recipient {
            address: Address::generate(&env),
            percentage: 20,
        },
    ];
    client.create_listing(
        &artist,
        &cid,
        &1_000_000_i128,
        &symbol_short!("XLM"),
        &recipients,
    );
}

// ── cancel_listing ───────────────────────────────────────────

#[test]
fn test_cancel_listing_success() {
    let (env, client, artist, _, _) = setup();
    let cid = bytes!(&env, 0x516d74657374);
    let id = client.create_listing(
        &artist,
        &cid,
        &5_000_000_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );

    let result = client.cancel_listing(&artist, &id);
    assert!(result);

    let listing = client.get_listing(&id);
    assert_eq!(listing.status, ListingStatus::Cancelled);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_cancel_listing_wrong_artist() {
    let (env, client, artist, buyer, _) = setup();
    let cid = bytes!(&env, 0x516d74657374);
    let id = client.create_listing(
        &artist,
        &cid,
        &5_000_000_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );
    client.cancel_listing(&buyer, &id);
}

// ── get_artist_listings ──────────────────────────────────────

#[test]
fn test_get_artist_listings() {
    let (env, client, artist, _, _) = setup();
    let cid = bytes!(&env, 0x516d74657374);

    client.create_listing(
        &artist,
        &cid,
        &1_000_000_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );
    client.create_listing(
        &artist,
        &cid,
        &2_000_000_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );
    client.create_listing(
        &artist,
        &cid,
        &3_000_000_i128,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );

    let ids = client.get_artist_listings(&artist);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.get(0).unwrap(), 1_u64);
    assert_eq!(ids.get(1).unwrap(), 2_u64);
    assert_eq!(ids.get(2).unwrap(), 3_u64);
}

#[test]
fn test_buy_artwork_success() {
    let (env, client, artist, buyer, _) = setup();

    let cid = bytes!(&env, 0x516d74657374);
    let price = 10_000_000_i128;
    let id = client.create_listing(
        &artist,
        &cid,
        &price,
        &symbol_short!("XLM"),
        &valid_recipients(&env, &artist),
    );

    let result = client.buy_artwork(&buyer, &id);
    assert!(result);

    let listing = client.get_listing(&id);
    assert_eq!(listing.status, ListingStatus::Sold);
    assert_eq!(listing.owner, Some(buyer));
}

#[test]
fn test_buy_artwork_complex_split() {
    let (env, client, artist, buyer, _) = setup();

    let colab1 = Address::generate(&env);
    let colab2 = Address::generate(&env);

    let cid = bytes!(&env, 0x516d74657374);
    let price = 10_000_000_i128; // 1 XLM

    // test precision rounding 33/33/34
    let recipients = vec![
        &env,
        Recipient {
            address: artist.clone(),
            percentage: 33,
        },
        Recipient {
            address: colab1.clone(),
            percentage: 33,
        },
        Recipient {
            address: colab2.clone(),
            percentage: 34, // Last receiver takes the exact fractional remainder securely
        },
    ];

    let id = client.create_listing(&artist, &cid, &price, &symbol_short!("XLM"), &recipients);
    assert!(client.buy_artwork(&buyer, &id));
}

// ── get_listing not found ────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_get_listing_not_found() {
    let (_env, client, _, _, _) = setup();
    client.get_listing(&999);
}
