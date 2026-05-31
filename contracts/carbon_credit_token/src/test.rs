#![cfg(test)]

use crate::{CarbonCreditToken, CarbonCreditTokenClient};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, Address, Bytes, Env, String,
};

#[contract]
pub struct MockRbacContract;

#[contractimpl]
impl MockRbacContract {
    pub fn has_role(_env: Env, _address: Address, _role: String) -> bool {
        true // Simplest mock: everyone has every role
    }
}

fn setup_env<'a>() -> (Env, CarbonCreditTokenClient<'a>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let rbac_id = env.register_contract(None, MockRbacContract);
    let token_id = env.register_contract(None, CarbonCreditToken);
    let client = CarbonCreditTokenClient::new(&env, &token_id);

    client.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "Carbon Credit Token"),
        &String::from_str(&env, "CCT"),
        &0u32,
        &String::from_str(&env, "Amazon Reforestation"),
        &String::from_str(&env, "2023"),
        &String::from_str(&env, "Brazil"),
        &String::from_str(&env, "https://farmcredit.xyz/amazon-1"),
    );

    let verifier = Address::generate(&env);
    let user = Address::generate(&env);

    (env, client, admin, verifier, user)
}

#[test]
fn test_initialize() {
    let (_, token, _, _, _) = setup_env();

    assert_eq!(token.name(), String::from_str(&token.env, "Carbon Credit Token"));
    assert_eq!(token.symbol(), String::from_str(&token.env, "CCT"));
    assert_eq!(token.decimals(), 0u32);
    assert_eq!(token.total_supply(), 0i128);
    assert_eq!(token.total_retired(), 0i128);
}

#[test]
fn test_retire_and_certificate_issuance() {
    let (env, token, _, verifier, user) = setup_env();

    let hash1 = Bytes::from_slice(&env, b"report_hash_1");
    let hash2 = Bytes::from_slice(&env, b"report_hash_2");
    let methodology = String::from_str(&env, "VCS");

    // Mint some tokens
    token.mint(&verifier, &user, &1000, &hash1);
    assert_eq!(token.balance(&user), 1000);

    // Retire some tokens
    token.retire(&user, &300, &hash2, &methodology);

    assert_eq!(token.balance(&user), 700);
    assert_eq!(token.total_retired(), 300);

    // Verify NFT creation
    assert_eq!(token.certificate_count(), 1);
    let cert = token.get_certificate(&1).unwrap();
    assert_eq!(cert.owner, user);
    assert_eq!(cert.amount, 300);
    assert_eq!(cert.project_name, String::from_str(&env, "Amazon Reforestation"));
    assert_eq!(cert.vintage, String::from_str(&env, "2023"));
    assert_eq!(cert.location, String::from_str(&env, "Brazil"));
    assert_eq!(cert.metadata_url, String::from_str(&env, "https://farmcredit.xyz/amazon-1"));
}

#[test]
fn test_multiple_retirements() {
    let (env, token, _, verifier, user) = setup_env();

    let hash1 = Bytes::from_slice(&env, b"h1");
    let hash2 = Bytes::from_slice(&env, b"h2");
    let hash3 = Bytes::from_slice(&env, b"h3");
    let methodology = String::from_str(&env, "VCS");

    token.mint(&verifier, &user, &1000, &hash1);
    
    token.retire(&user, &100, &hash2, &methodology);
    token.retire(&user, &200, &hash3, &methodology);

    assert_eq!(token.certificate_count(), 2);
    assert_eq!(token.get_certificate(&1).unwrap().amount, 100);
    assert_eq!(token.get_certificate(&2).unwrap().amount, 200);
    assert_eq!(token.total_retired(), 300);
}
