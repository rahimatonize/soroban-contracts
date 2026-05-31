use soroban_sdk::{contractclient, Address, Env, String};

use crate::storage::read_rbac_contract;

/// The role string that grants minting authority.
/// Must exactly match the role name registered in the RBAC contract.
pub const VERIFIER_ROLE: &str = "Verifier";

/// Cross-contract client interface for the external RBAC contract.
///
/// The RBAC contract must expose a `has_role(address, role) -> bool`
/// entry-point. Any contract that satisfies this interface can be used
/// as the authority source — enabling the token contract to remain
/// agnostic about the RBAC implementation details.
#[contractclient(name = "RbacContractClient")]
#[allow(dead_code)]
pub trait RbacContractInterface {
    /// Returns `true` when `address` holds `role` in the RBAC registry.
    fn has_role(env: Env, address: Address, role: String) -> bool;
}

/// Asserts that `caller` both:
///   1. Signed the current transaction (`require_auth`), and
///   2. Holds the `Verifier` role in the registered RBAC contract.
///
/// Panics with a descriptive message if either check fails, preventing
/// any state mutation in the calling function.
pub fn require_verifier(e: &Env, caller: &Address) {
    // Ensure the address actually authorised this invocation.
    caller.require_auth();

    let rbac_id = read_rbac_contract(e);
    let client = RbacContractClient::new(e, &rbac_id);
    let role = String::from_str(e, VERIFIER_ROLE);

    if !client.has_role(caller, &role) {
        panic!(
            "minting rejected: address does not hold the '{}' role",
            VERIFIER_ROLE
        );
    }
}
