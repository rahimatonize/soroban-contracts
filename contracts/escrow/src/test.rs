#![cfg(test)]

use crate::{EscrowContract, EscrowContractClient, Offer};
use carbon_credit_token::{CarbonCreditToken, CarbonCreditTokenClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Bytes, Env, String,
};

// ── Mock RBAC (grants verifier role to everyone) ─────────────────────────────

#[soroban_sdk::contract]
pub struct MockRbac;

#[soroban_sdk::contractimpl]
impl MockRbac {
    pub fn has_role(_env: Env, _address: Address, _role: String) -> bool {
        true
    }
}

// ── Test helpers ──────────────────────────────────────────────────────────────

struct Setup<'a> {
    env: Env,
    escrow: EscrowContractClient<'a>,
    escrow_id: Address,
    carbon: CarbonCreditTokenClient<'a>,
    carbon_id: Address,
    usdc: CarbonCreditTokenClient<'a>,
    usdc_id: Address,
    admin: Address,
    seller: Address,
    buyer: Address,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, MockRbac);
    let admin = Address::generate(&env);

    // Carbon token
    let carbon_id = env.register_contract(None, CarbonCreditToken);
    let carbon = CarbonCreditTokenClient::new(&env, &carbon_id);
    carbon.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "Carbon Credit"),
        &String::from_str(&env, "CCT"),
        &0u32,
    );

    // USDC token (reuse same contract type)
    let usdc_id = env.register_contract(None, CarbonCreditToken);
    let usdc = CarbonCreditTokenClient::new(&env, &usdc_id);
    usdc.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "USD Coin"),
        &String::from_str(&env, "USDC"),
        &6u32,
    );

    // Escrow
    let escrow_id = env.register_contract(None, EscrowContract);
    let escrow = EscrowContractClient::new(&env, &escrow_id);
    escrow.initialize();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Fund seller with carbon, buyer with USDC
    let hash1 = Bytes::from_slice(&env, b"seller_mint_hash");
    let hash2 = Bytes::from_slice(&env, b"buyer_mint_hash_");
    carbon.mint(&admin, &seller, &10_000, &hash1);
    usdc.mint(&admin, &buyer, &100_000, &hash2);

    Setup {
        env,
        escrow,
        escrow_id,
        carbon,
        carbon_id,
        usdc,
        usdc_id,
        admin,
        seller,
        buyer,
    }
}

// ── Initialization ────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    client.initialize(); // no panic = pass
}

#[test]
fn test_initialize_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    client.initialize();
    assert!(client.try_initialize().is_err());
}

#[test]
fn test_create_offer_happy_path() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);

    assert_eq!(offer_id, 1);
    // Seller's carbon reduced, escrow holds it
    assert_eq!(s.carbon.balance(&s.seller), 9_000);
    assert_eq!(s.carbon.balance(&s.escrow_id), 1_000);

    let offer = s.escrow.get_offer(&offer_id).unwrap();
    assert_eq!(offer.carbon_amount, 1000);
    assert_eq!(offer.usdc_amount, 5000);
    assert!(!offer.is_cancelled);
}

#[test]
fn test_create_offer_zero_carbon_panics() {
    let s = setup();
    assert!(s
        .escrow
        .try_create_offer(&s.seller, &0, &5000, &s.carbon_id, &s.usdc_id, &100)
        .is_err());
}

#[test]
fn test_create_offer_zero_usdc_panics() {
    let s = setup();
    assert!(s
        .escrow
        .try_create_offer(&s.seller, &1000, &0, &s.carbon_id, &s.usdc_id, &100)
        .is_err());
}

#[test]
fn test_create_offer_negative_carbon_panics() {
    let s = setup();
    assert!(s
        .escrow
        .try_create_offer(&s.seller, &-100, &5000, &s.carbon_id, &s.usdc_id, &100)
        .is_err());
}

#[test]
fn test_create_offer_emits_event() {
    let s = setup();
    s.escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    // At least one event was emitted (offer_created + token transfer events)
    assert!(!s.env.events().all().is_empty());
}

#[test]
fn test_fill_offer_emits_event() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    let events_before = s.env.events().all().len();
    s.escrow.fill_offer(&offer_id, &s.buyer, &500);
    assert!(s.env.events().all().len() > events_before);
}

#[test]
fn test_cancel_offer_emits_event() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    let events_before = s.env.events().all().len();
    s.escrow.cancel_offer(&offer_id, &s.seller);
    assert!(s.env.events().all().len() > events_before);
}

// ── Full fill ─────────────────────────────────────────────────────────────────

#[test]
fn test_full_fill_happy_path() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);

    s.escrow.fill_offer(&offer_id, &s.buyer, &1000);

    // Buyer receives carbon, seller receives USDC
    assert_eq!(s.carbon.balance(&s.buyer), 1000);
    assert_eq!(s.usdc.balance(&s.seller), 5000);
    assert_eq!(s.usdc.balance(&s.buyer), 95_000);
    assert_eq!(s.carbon.balance(&s.escrow_id), 0);

    // Fully filled offer is removed
    assert!(s.escrow.get_offer(&offer_id).is_none());
}

// ── Partial fill ──────────────────────────────────────────────────────────────

#[test]
fn test_partial_fill() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);

    s.escrow.fill_offer(&offer_id, &s.buyer, &400);

    assert_eq!(s.carbon.balance(&s.buyer), 400);
    assert_eq!(s.usdc.balance(&s.seller), 2000); // 400/1000 * 5000
    assert_eq!(s.carbon.balance(&s.escrow_id), 600);

    let offer = s.escrow.get_offer(&offer_id).unwrap();
    assert_eq!(offer.remaining_carbon(), 600);
    assert_eq!(offer.remaining_usdc(), 3000);
}

// ── Multiple partial fills ────────────────────────────────────────────────────

#[test]
fn test_multiple_partial_fills() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);

    s.escrow.fill_offer(&offer_id, &s.buyer, &300); // fill 1
    s.escrow.fill_offer(&offer_id, &s.buyer, &400); // fill 2
    s.escrow.fill_offer(&offer_id, &s.buyer, &300); // fill 3 — completes offer

    assert_eq!(s.carbon.balance(&s.buyer), 1000);
    assert_eq!(s.usdc.balance(&s.seller), 5000);
    assert_eq!(s.carbon.balance(&s.escrow_id), 0);
    assert!(s.escrow.get_offer(&offer_id).is_none());
}

// ── Cancel offer ──────────────────────────────────────────────────────────────

#[test]
fn test_cancel_offer_returns_tokens() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);

    s.escrow.cancel_offer(&offer_id, &s.seller);

    assert_eq!(s.carbon.balance(&s.seller), 10_000); // fully returned
    assert_eq!(s.carbon.balance(&s.escrow_id), 0);

    let offer = s.escrow.get_offer(&offer_id).unwrap();
    assert!(offer.is_cancelled);
}

#[test]
fn test_cancel_after_partial_fill() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);

    s.escrow.fill_offer(&offer_id, &s.buyer, &300);
    s.escrow.cancel_offer(&offer_id, &s.seller);

    // Seller gets back remaining 700 carbon
    assert_eq!(s.carbon.balance(&s.seller), 9_700);
    assert_eq!(s.carbon.balance(&s.escrow_id), 0);
}

// ── Error cases ───────────────────────────────────────────────────────────────

#[test]
fn test_fill_nonexistent_offer_panics() {
    let s = setup();
    assert!(s.escrow.try_fill_offer(&999, &s.buyer, &100).is_err());
}

#[test]
fn test_fill_zero_amount_panics() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    assert!(s.escrow.try_fill_offer(&offer_id, &s.buyer, &0).is_err());
}

#[test]
fn test_fill_exceeds_remaining_panics() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    assert!(s.escrow.try_fill_offer(&offer_id, &s.buyer, &1001).is_err());
}

#[test]
fn test_fill_cancelled_offer_panics() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    s.escrow.cancel_offer(&offer_id, &s.seller);
    assert!(s.escrow.try_fill_offer(&offer_id, &s.buyer, &100).is_err());
}

#[test]
fn test_cancel_nonexistent_offer_panics() {
    let s = setup();
    assert!(s.escrow.try_cancel_offer(&999, &s.seller).is_err());
}

#[test]
fn test_cancel_by_non_seller_panics() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    let non_seller = Address::generate(&s.env);
    assert!(s.escrow.try_cancel_offer(&offer_id, &non_seller).is_err());
}

#[test]
fn test_cancel_already_cancelled_panics() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &100);
    s.escrow.cancel_offer(&offer_id, &s.seller);
    assert!(s.escrow.try_cancel_offer(&offer_id, &s.seller).is_err());
}

// ── View functions ────────────────────────────────────────────────────────────

#[test]
fn test_get_nonexistent_offer_returns_none() {
    let s = setup();
    assert!(s.escrow.get_offer(&999).is_none());
}

#[test]
fn test_get_remaining_amount_nonexistent_offer() {
    let s = setup();
    let (carbon, usdc) = s.escrow.get_remaining_amount(&999);
    assert_eq!(carbon, 0);
    assert_eq!(usdc, 0);
}

// ── Offer struct unit tests ───────────────────────────────────────────────────

#[test]
fn test_offer_remaining_and_fully_filled() {
    let env = Env::default();
    let mut offer = Offer {
        offer_id: 1,
        seller: Address::generate(&env),
        carbon_amount: 1000,
        usdc_amount: 5000,
        filled_carbon: 300,
        filled_usdc: 1500,
        carbon_token: Address::generate(&env),
        usdc_token: Address::generate(&env),
        is_cancelled: false,
        min_fill_amount: 50,
    };

    assert_eq!(offer.remaining_carbon(), 700);
    assert_eq!(offer.remaining_usdc(), 3500);
    assert!(!offer.is_fully_filled());

    offer.filled_carbon = 1000;
    offer.filled_usdc = 5000;
    assert!(offer.is_fully_filled());
}

// ── Rounding ──────────────────────────────────────────────────────────────────

#[test]
fn test_partial_fill_rounds_up_in_favor_of_seller() {
    let s = setup();
    // 3 carbon for 10_000 USDC; filling 1 carbon = 3333.33... → should round up to 3334
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &3, &10_000, &s.carbon_id, &s.usdc_id, &1);
    s.escrow.fill_offer(&offer_id, &s.buyer, &1);
    assert_eq!(s.usdc.balance(&s.seller), 3334);
}

// ── Authorization ─────────────────────────────────────────────────────────────
// Note: auth failures in soroban abort the process and cannot be caught with try_* methods.

// ── min_fill_amount ───────────────────────────────────────────────────────────

#[test]
fn test_create_offer_zero_min_fill_panics() {
    let s = setup();
    assert!(s
        .escrow
        .try_create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &0)
        .is_err());
}

#[test]
fn test_fill_below_minimum_panics() {
    let s = setup();
    // min_fill_amount = 200; filling 100 (< 200) with 900 remaining should fail
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &200);
    assert!(s.escrow.try_fill_offer(&offer_id, &s.buyer, &100).is_err());
}

#[test]
fn test_final_fill_below_minimum_allowed() {
    let s = setup();
    // min_fill_amount = 200; after filling 900, only 100 remains — final fill allowed
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &200);
    s.escrow.fill_offer(&offer_id, &s.buyer, &900);
    // 100 remaining < min_fill_amount=200, but it's the full remainder — must succeed
    s.escrow.fill_offer(&offer_id, &s.buyer, &100);
    assert!(s.escrow.get_offer(&offer_id).is_none());
}

#[test]
fn test_fill_exactly_minimum_succeeds() {
    let s = setup();
    let offer_id = s
        .escrow
        .create_offer(&s.seller, &1000, &5000, &s.carbon_id, &s.usdc_id, &200);
    s.escrow.fill_offer(&offer_id, &s.buyer, &200);
    assert_eq!(s.carbon.balance(&s.buyer), 200);
}
