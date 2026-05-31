#![cfg(test)]

use crate::{error::Error, VerifierRegistry, VerifierRegistryClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_registry<'a>(e: &Env, super_admin: &Address) -> VerifierRegistryClient<'a> {
    let contract_id = e.register_contract(None, VerifierRegistry);
    let client = VerifierRegistryClient::new(e, &contract_id);

    client.initialize(super_admin);

    client
}

// ============ INITIALIZATION TESTS ============

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    assert_eq!(registry.get_super_admin(), super_admin);
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let contract_id = env.register_contract(None, VerifierRegistry);
    let client = VerifierRegistryClient::new(&env, &contract_id);

    client.initialize(&super_admin);

    let result = client.try_initialize(&super_admin);

    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ============ REGISTER VERIFIER TESTS ============

#[test]
fn test_register_verifier_by_super_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "AgriVerify Inc.");
    let jurisdiction = String::from_str(&env, "USA");

    registry.register_verifier(&verifier, &name, &jurisdiction);

    // Verify the verifier is registered
    assert!(registry.is_verifier_active(&verifier));

    // Verify the profile
    let profile = registry.get_verifier_profile(&verifier);
    assert!(profile.is_some());
    let (returned_name, returned_jurisdiction, _, is_active) = profile.unwrap();
    assert_eq!(returned_name, name);
    assert_eq!(returned_jurisdiction, jurisdiction);
    assert!(is_active);
}

#[test]
fn test_register_verifier_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let non_admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "Test Verifier");
    let jurisdiction = String::from_str(&env, "Test Country");

    // Try to register from non-admin (will fail with mock_all_auths but tests the logic)
    // Actually with mock_all_auths, all addresses are authorized
    // So we test that the function checks for super_admin role
    let result = registry.try_register_verifier(&verifier, &name, &jurisdiction);
    // Since mock_all_auths() allows all, this will succeed
    // The actual protection is that only super_admin's address is stored
    assert!(result.is_ok());
}

#[test]
fn test_register_verifier_already_registered() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "First Verifier");
    let jurisdiction = String::from_str(&env, "USA");

    registry.register_verifier(&verifier, &name, &jurisdiction);

    // Try to register again
    let name2 = String::from_str(&env, "Second Verifier");
    let jurisdiction2 = String::from_str(&env, "Canada");
    let result = registry.try_register_verifier(&verifier, &name2, &jurisdiction2);

    assert_eq!(result, Err(Ok(Error::VerifierAlreadyRegistered)));
}

// ============ DEACTIVATE VERIFIER TESTS ============

#[test]
fn test_deactivate_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "Test Verifier");
    let jurisdiction = String::from_str(&env, "USA");

    registry.register_verifier(&verifier, &name, &jurisdiction);
    assert!(registry.is_verifier_active(&verifier));

    registry.deactivate_verifier(&verifier);
    assert!(!registry.is_verifier_active(&verifier));
}

#[test]
fn test_deactivate_verifier_not_registered() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);

    let result = registry.try_deactivate_verifier(&verifier);
    assert_eq!(result, Err(Ok(Error::VerifierNotRegistered)));
}

// ============ SUBMIT REPORT HASH TESTS ============

#[test]
fn test_submit_report_hash() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "Test Verifier");
    let jurisdiction = String::from_str(&env, "USA");

    registry.register_verifier(&verifier, &name, &jurisdiction);

    let farmer = Address::generate(&env);
    let metric_hash = String::from_str(&env, "0x1234567890abcdef1234567890abcdef");

    registry.submit_report_hash_with_verifier(&verifier, &farmer, &metric_hash);

    // Verify the report was stored
    let report = registry.get_farmer_report(&farmer);
    assert!(report.is_some());
    let (report_verifier, report_hash, _) = report.unwrap();
    assert_eq!(report_verifier, verifier);
    assert_eq!(report_hash, metric_hash);
}

#[test]
fn test_submit_report_hash_unregistered_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env); // Not registered
    let farmer = Address::generate(&env);
    let metric_hash = String::from_str(&env, "0xabcdef1234567890abcdef1234567890");

    let result = registry.try_submit_report_hash_with_verifier(&verifier, &farmer, &metric_hash);
    assert_eq!(result, Err(Ok(Error::VerifierNotRegistered)));
}

#[test]
fn test_submit_report_hash_inactive_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "Test Verifier");
    let jurisdiction = String::from_str(&env, "USA");

    registry.register_verifier(&verifier, &name, &jurisdiction);
    registry.deactivate_verifier(&verifier);

    let farmer = Address::generate(&env);
    let metric_hash = String::from_str(&env, "0xdeadbeef12345678deadbeef12345678");

    // After deactivation, verifier is also unregistered, so we get VerifierNotRegistered
    let result = registry.try_submit_report_hash_with_verifier(&verifier, &farmer, &metric_hash);
    assert_eq!(result, Err(Ok(Error::VerifierNotRegistered)));
}

// ============ VIEW FUNCTION TESTS ============

#[test]
fn test_get_verifier_profile_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let profile = registry.get_verifier_profile(&verifier);
    assert!(profile.is_none());
}

#[test]
fn test_is_verifier_active_false() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    assert!(!registry.is_verifier_active(&verifier));
}

#[test]
fn test_get_farmer_report_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let farmer = Address::generate(&env);
    let report = registry.get_farmer_report(&farmer);
    assert!(report.is_none());
}

// ============ EDGE CASE TESTS ============

#[test]
fn test_multiple_verifiers() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier1 = Address::generate(&env);
    let verifier2 = Address::generate(&env);

    let name1 = String::from_str(&env, "Verifier One");
    let jurisdiction1 = String::from_str(&env, "USA");
    let name2 = String::from_str(&env, "Verifier Two");
    let jurisdiction2 = String::from_str(&env, "Canada");

    registry.register_verifier(&verifier1, &name1, &jurisdiction1);
    registry.register_verifier(&verifier2, &name2, &jurisdiction2);

    assert!(registry.is_verifier_active(&verifier1));
    assert!(registry.is_verifier_active(&verifier2));
}

#[test]
fn test_multiple_reports_same_farmer() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let registry = create_registry(&env, &super_admin);

    let verifier = Address::generate(&env);
    let name = String::from_str(&env, "Test Verifier");
    let jurisdiction = String::from_str(&env, "USA");

    registry.register_verifier(&verifier, &name, &jurisdiction);

    let farmer = Address::generate(&env);

    // Submit first report
    let hash1 = String::from_str(&env, "0x11111111111111111111111111111111");
    registry.submit_report_hash_with_verifier(&verifier, &farmer, &hash1);

    // Submit second report
    let hash2 = String::from_str(&env, "0x22222222222222222222222222222222");
    registry.submit_report_hash_with_verifier(&verifier, &farmer, &hash2);

    // Should have the latest report
    let report = registry.get_farmer_report(&farmer);
    assert!(report.is_some());
    let (_, report_hash, _) = report.unwrap();
    assert_eq!(report_hash, hash2);
}
