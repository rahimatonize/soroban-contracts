#![no_std]

mod error;
mod storage;

use error::Error;
use soroban_sdk::{contract, contractimpl, Address, Env, String};

use storage::{
    is_initialized, is_verifier_registered, read_report, read_super_admin, read_verifier_profile,
    register_verifier, set_initialized, unregister_verifier, write_report, write_super_admin,
    ReportData, VerifierProfile, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};

#[contract]
pub struct VerifierRegistry;

#[contractimpl]
impl VerifierRegistry {
    /// Initializes the contract with the SuperAdmin.
    /// Can only be called once.
    pub fn initialize(env: Env, super_admin: Address) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_initialized(&env);
        write_super_admin(&env, &super_admin);

        Ok(())
    }

    // ── Admin functions (SuperAdmin only) ─────────────────────────────────────────

    /// Registers a new verifier with their profile.
    /// Only the SuperAdmin can call this.
    pub fn register_verifier(
        env: Env,
        verifier: Address,
        name: String,
        jurisdiction: String,
    ) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        // Extend TTL for storage
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Check if verifier is already registered
        if is_verifier_registered(&env, &verifier) {
            return Err(Error::VerifierAlreadyRegistered);
        }

        let profile = VerifierProfile {
            name,
            registration_date: env.ledger().sequence(),
            jurisdiction,
            is_active: true,
        };

        storage::write_verifier_profile(&env, &verifier, &profile);
        register_verifier(&env, &verifier);

        Ok(())
    }

    /// Deactivates a verifier (soft delete - keeps profile for audit trail).
    /// Only the SuperAdmin can call this.
    pub fn deactivate_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        // Extend TTL for storage
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Check if verifier is registered
        if !is_verifier_registered(&env, &verifier) {
            return Err(Error::VerifierNotRegistered);
        }

        // Update profile to mark as inactive
        if let Some(mut profile) = read_verifier_profile(&env, &verifier) {
            profile.is_active = false;
            storage::write_verifier_profile(&env, &verifier, &profile);
        }

        unregister_verifier(&env, &verifier);

        Ok(())
    }

    // ── Verifier functions ─────────────────────────────────────────────────────────

    /// Submits a report hash for a farmer.
    /// Only registered and active verifiers can call this.
    pub fn submit_report_hash(
        env: Env,
        verifier: Address,
        farmer: Address,
        metric_hash: String,
    ) -> Result<(), Error> {
        verifier.require_auth();

        if !is_verifier_registered(&env, &verifier) {
            return Err(Error::VerifierNotRegistered);
        }

        if let Some(profile) = read_verifier_profile(&env, &verifier) {
            if !profile.is_active {
                return Err(Error::Unauthorized);
            }
        } else {
            return Err(Error::VerifierNotRegistered);
        }

        let report: ReportData = (
            verifier.clone(),
            metric_hash.clone(),
            env.ledger().sequence(),
        );
        write_report(&env, &farmer, &report);

        env.events().publish(
            (soroban_sdk::symbol_short!("rpt_sub"),),
            (verifier, farmer, metric_hash),
        );

        Ok(())
    }

    /// Helper to submit report with explicit verifier (for testing/Integration)
    pub fn submit_report_hash_with_verifier(
        env: Env,
        verifier: Address,
        farmer: Address,
        metric_hash: String,
    ) -> Result<(), Error> {
        // Verify the verifier is registered
        if !is_verifier_registered(&env, &verifier) {
            return Err(Error::VerifierNotRegistered);
        }

        // Verify the verifier is active
        if let Some(profile) = read_verifier_profile(&env, &verifier) {
            if !profile.is_active {
                return Err(Error::Unauthorized);
            }
        } else {
            return Err(Error::VerifierNotRegistered);
        }

        // Create and store the report record
        let report: ReportData = (verifier, metric_hash, env.ledger().sequence());
        write_report(&env, &farmer, &report);

        Ok(())
    }

    // ── View functions ───────────────────────────────────────────────────────────

    /// Returns the verifier profile if registered.
    pub fn get_verifier_profile(
        env: Env,
        verifier: Address,
    ) -> Option<(String, String, u32, bool)> {
        read_verifier_profile(&env, &verifier)
            .map(|p| (p.name, p.jurisdiction, p.registration_date, p.is_active))
    }

    /// Returns true if the verifier is registered and active.
    pub fn is_verifier_active(env: Env, verifier: Address) -> bool {
        if let Some(profile) = read_verifier_profile(&env, &verifier) {
            profile.is_active && is_verifier_registered(&env, &verifier)
        } else {
            false
        }
    }

    /// Returns the latest report for a farmer: (verifier, metric_hash, submission_ledger)
    pub fn get_farmer_report(env: Env, farmer: Address) -> Option<(Address, String, u32)> {
        read_report(&env, &farmer)
    }

    /// Returns the SuperAdmin address.
    pub fn get_super_admin(env: Env) -> Address {
        read_super_admin(&env)
    }
}

mod test;
