/// Integration tests: real RbacContract + CarbonCreditToken in the same env.
///
/// These tests complement the mock-based unit tests in `test.rs` by exercising
/// the actual cross-contract call path that `require_verifier` takes at runtime.
/// Any serialisation mismatch, storage-key collision, or interface drift between
/// the two contracts will surface here rather than silently passing with a mock.
#[cfg(test)]
mod integration_tests {
    use crate::{CarbonCreditToken, CarbonCreditTokenClient};
    use rbac::{RbacContract, RbacContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Deploys and initialises both contracts, returning a ready-to-use tuple.
    ///
    /// The RBAC contract is initialised with `super_admin` as its SuperAdmin.
    /// The token contract is wired to the real RBAC contract address.
    fn setup<'a>() -> (
        Env,
        CarbonCreditTokenClient<'a>,
        RbacContractClient<'a>,
        Address, // super_admin
    ) {
        use soroban_sdk::String;

        let env = Env::default();
        env.mock_all_auths();

        let super_admin = Address::generate(&env);

        // Deploy the real RBAC contract.
        let rbac_id = env.register_contract(None, RbacContract);
        let rbac = RbacContractClient::new(&env, &rbac_id);
        rbac.initialize(&super_admin);

        // Deploy the token contract wired to the real RBAC contract.
        let token_id = env.register_contract(None, CarbonCreditToken);
        let token = CarbonCreditTokenClient::new(&env, &token_id);
        token.initialize(
            &super_admin,
            &rbac_id,
            &String::from_str(&env, "Carbon Credit Token"),
            &String::from_str(&env, "CCT"),
            &0u32,
            &String::from_str(&env, "Amazon Reforestation"),
            &String::from_str(&env, "2023"),
            &String::from_str(&env, "Brazil"),
            &String::from_str(&env, "https://farmcredit.xyz/amazon-1"),
        );

        (env, token, rbac, super_admin)
    }

    fn hash(env: &Env, tag: &[u8]) -> Bytes {
        Bytes::from_slice(env, tag)
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    /// A verifier granted through the real RBAC contract can mint tokens.
    #[test]
    fn test_verifier_can_mint() {
        let (env, token, rbac, super_admin) = setup();
        let verifier = Address::generate(&env);
        let recipient = Address::generate(&env);

        rbac.grant_verifier(&super_admin, &verifier);

        token.mint(&verifier, &recipient, &500, &hash(&env, b"hash-001"));

        assert_eq!(token.balance(&recipient), 500);
        assert_eq!(token.total_supply(), 500);
    }

    /// An address with no role must not be able to mint.
    #[test]
    #[should_panic]
    fn test_non_verifier_cannot_mint() {
        let (env, token, _rbac, _super_admin) = setup();
        let stranger = Address::generate(&env);
        let recipient = Address::generate(&env);

        // No role granted — `has_role` returns false, `require_verifier` panics.
        token.mint(&stranger, &recipient, &100, &hash(&env, b"hash-002"));
    }

    /// Revoking the Verifier role through RBAC immediately prevents minting.
    #[test]
    #[should_panic]
    fn test_revoked_verifier_cannot_mint() {
        let (env, token, rbac, super_admin) = setup();
        let verifier = Address::generate(&env);
        let recipient = Address::generate(&env);

        // Grant, mint once successfully, then revoke.
        rbac.grant_verifier(&super_admin, &verifier);
        token.mint(&verifier, &recipient, &200, &hash(&env, b"hash-003"));
        assert_eq!(token.balance(&recipient), 200);

        rbac.revoke_role(&super_admin, &verifier);

        // Second mint must panic — role has been revoked.
        token.mint(&verifier, &recipient, &100, &hash(&env, b"hash-004"));
    }

    /// An Admin in RBAC does not automatically gain Verifier minting rights.
    #[test]
    #[should_panic]
    fn test_admin_role_does_not_grant_mint() {
        let (env, token, rbac, super_admin) = setup();
        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);

        rbac.grant_admin(&super_admin, &admin);

        // Admin ≠ Verifier — minting must be rejected.
        token.mint(&admin, &recipient, &100, &hash(&env, b"hash-005"));
    }

    /// The SuperAdmin of RBAC does not automatically gain Verifier minting rights.
    #[test]
    #[should_panic]
    fn test_super_admin_cannot_mint_without_verifier_role() {
        let (env, token, _rbac, super_admin) = setup();
        let recipient = Address::generate(&env);

        // SuperAdmin has no Verifier role — minting must be rejected.
        token.mint(&super_admin, &recipient, &100, &hash(&env, b"hash-006"));
    }

    /// Multiple distinct verifiers can each mint independently.
    #[test]
    fn test_multiple_verifiers_can_mint_independently() {
        let (env, token, rbac, super_admin) = setup();
        let verifier_a = Address::generate(&env);
        let verifier_b = Address::generate(&env);
        let recipient = Address::generate(&env);

        rbac.grant_verifier(&super_admin, &verifier_a);
        rbac.grant_verifier(&super_admin, &verifier_b);

        token.mint(&verifier_a, &recipient, &300, &hash(&env, b"hash-007"));
        token.mint(&verifier_b, &recipient, &700, &hash(&env, b"hash-008"));

        assert_eq!(token.balance(&recipient), 1000);
        assert_eq!(token.total_supply(), 1000);
    }

    /// Revoking one verifier does not affect another verifier's ability to mint.
    #[test]
    fn test_revoking_one_verifier_does_not_affect_another() {
        let (env, token, rbac, super_admin) = setup();
        let verifier_a = Address::generate(&env);
        let verifier_b = Address::generate(&env);
        let recipient = Address::generate(&env);

        rbac.grant_verifier(&super_admin, &verifier_a);
        rbac.grant_verifier(&super_admin, &verifier_b);

        rbac.revoke_role(&super_admin, &verifier_a);

        // verifier_b must still be able to mint.
        token.mint(&verifier_b, &recipient, &400, &hash(&env, b"hash-009"));
        assert_eq!(token.balance(&recipient), 400);
    }

    /// The same report hash cannot be used twice, even by a valid verifier.
    #[test]
    #[should_panic]
    fn test_duplicate_report_hash_rejected() {
        let (env, token, rbac, super_admin) = setup();
        let verifier = Address::generate(&env);
        let recipient = Address::generate(&env);

        rbac.grant_verifier(&super_admin, &verifier);

        let h = hash(&env, b"hash-010");
        token.mint(&verifier, &recipient, &100, &h);

        // Second mint with the same hash must panic.
        token.mint(&verifier, &recipient, &100, &h);
    }

    /// `rbac_contract()` on the token returns the address of the real RBAC contract.
    #[test]
    fn test_rbac_contract_address_stored_correctly() {
        let (_env, token, rbac, _super_admin) = setup();
        assert_eq!(token.rbac_contract(), *rbac.address);
    }
}
