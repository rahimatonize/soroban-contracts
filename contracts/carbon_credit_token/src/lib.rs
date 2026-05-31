#![no_std]

mod admin;
mod allowance;
mod balance;
mod certificate;
mod error;
mod events;
mod metadata;
mod rbac;
mod retirement;
mod storage;

#[cfg(test)]
mod test;

#[cfg(test)]
mod integration_test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::admin::{
    blacklist_address, grant_verifier, is_blacklisted, is_verifier, read_administrator,
    read_super_admin, revoke_verifier, unblacklist_address, write_administrator, write_super_admin,
};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::certificate::{
    increment_next_certificate_id, read_certificate, read_next_certificate_id, write_certificate,
    CertificateRecord,
};
use crate::retirement::{
    increment_next_retirement_id, read_retirement, write_retirement, RetirementRecord,
};
use crate::error::Error;
use crate::events::{
    ApproveEvent, BurnEvent, CertificateMintedEvent, MintEvent, RetirementEvent, TransferEvent,
};
use crate::metadata::{
    read_decimals, read_name, read_project_location, read_project_metadata_url, read_project_name,
    read_project_vintage, read_symbol, write_metadata, write_project_info,
};
use crate::rbac::require_verifier;
use crate::storage::{
    is_initialized, read_total_retired, read_total_supply, set_initialized, write_rbac_contract,
    write_total_retired, write_total_supply, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};

fn check_nonnegative_amount(amount: i128) -> Result<(), Error> {
    if amount < 0 {
        Err(Error::NegativeAmount)
    } else {
        Ok(())
    }
}

fn require_not_blacklisted(env: &Env, addr: &Address) -> Result<(), Error> {
    if is_blacklisted(env, addr) {
        Err(Error::Blacklisted)
    } else {
        Ok(())
    }
}

#[contract]
pub struct CarbonCreditToken;

#[contractimpl]
impl CarbonCreditToken {
    /// Initializes the contract with admin roles, RBAC address, and metadata.
    pub fn initialize(
        env: Env,
        admin: Address,
        rbac_contract: Address,
        name: String,
        symbol: String,
        decimals: u32,
        project_name: String,
        vintage: String,
        location: String,
        metadata_url: String,
    ) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_initialized(&env);
        write_administrator(&env, &admin);
        write_super_admin(&env, &admin);
        write_rbac_contract(&env, &rbac_contract);
        write_metadata(&env, name, symbol, decimals);
        write_total_supply(&env, 0);
        write_total_retired(&env, 0);
        write_project_info(&env, project_name, vintage, location, metadata_url);

        Ok(())
    }

    // ── RBAC management (SuperAdmin only) ────────────────────────────────────

    pub fn add_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        grant_verifier(&env, &verifier);
        Ok(())
    }

    pub fn remove_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        revoke_verifier(&env, &verifier);
        Ok(())
    }

    pub fn blacklist(env: Env, target: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        if target == super_admin {
            return Err(Error::CannotBlacklistSelf);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        blacklist_address(&env, &target);
        Ok(())
    }

    pub fn unblacklist(env: Env, target: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        unblacklist_address(&env, &target);
        Ok(())
    }

    pub fn add_to_blacklist_batch(env: Env, admin: Address, accounts: Vec<Address>) -> Result<(), Error> {
        admin.require_auth();
        if admin != read_super_admin(&env) {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        for account in accounts.iter() {
            if account == admin {
                return Err(Error::CannotBlacklistSelf);
            }
            blacklist_address(&env, &account);
        }

        Ok(())
    }

    pub fn remove_from_blacklist_batch(env: Env, admin: Address, accounts: Vec<Address>) -> Result<(), Error> {
        admin.require_auth();
        if admin != read_super_admin(&env) {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        for account in accounts.iter() {
            unblacklist_address(&env, &account);
        }

        Ok(())
    }

    pub fn transfer_super_admin(env: Env, successor: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        if successor == super_admin {
            return Err(Error::InvalidSuccessor);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_super_admin(&env, &successor);
        Ok(())
    }

    /// Proactively extends TTL for all critical instance storage. Admin only.
    pub fn extend_ttl(env: Env) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        Ok(())
    }

    // ── Token operations ──────────────────────────────────────────────────────

    pub fn mint(
        env: Env,
        verifier: Address,
        to: Address,
        amount: i128,
        report_hash: Bytes,
    ) -> Result<(), Error> {
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &verifier)?;
        require_not_blacklisted(&env, &to)?;

        require_verifier(&env, &verifier);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if crate::storage::is_report_hash_used(&env, &report_hash) {
            return Err(Error::ReportHashUsed);
        }
        crate::storage::mark_report_hash_used(&env, &report_hash);

        receive_balance(&env, to.clone(), amount);

        let new_supply = read_total_supply(&env) + amount;
        write_total_supply(&env, new_supply);

        MintEvent { to, amount }.publish(&env);
        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;
        require_not_blacklisted(&env, &to)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount)?;
        receive_balance(&env, to.clone(), amount);

        TransferEvent { from, to, amount }.publish(&env);
        Ok(())
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &spender)?;
        require_not_blacklisted(&env, &from)?;
        require_not_blacklisted(&env, &to)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&env, from.clone(), spender, amount)?;
        spend_balance(&env, from.clone(), amount)?;
        receive_balance(&env, to.clone(), amount);

        TransferEvent { from, to, amount }.publish(&env);
        Ok(())
    }

    /// Retires tokens and mints a Soulbound NFT certificate.
    pub fn retire(
        env: Env,
        from: Address,
        amount: i128,
        _report_hash: Bytes,
        _methodology: String,
        reason: String,
        beneficiary: String,
    ) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;

        if amount == 0 {
            return Err(Error::ZeroRetirementAmount);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount)?;

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        let new_retired = read_total_retired(&env) + amount;
        write_total_retired(&env, new_retired);

        let timestamp = env.ledger().timestamp();

        // Store on-chain retirement record
        let retirement_id = increment_next_retirement_id(&env);
        write_retirement(
            &env,
            RetirementRecord {
                id: retirement_id,
                retiree: from.clone(),
                amount,
                timestamp,
                reason,
                beneficiary,
            },
        );

        // Mint Soulbound NFT Certificate
        let cert_id = increment_next_certificate_id(&env);
        let cert = CertificateRecord {
            id: cert_id,
            owner: from.clone(),
            amount,
            timestamp,
            project_name: read_project_name(&env),
            vintage: read_project_vintage(&env),
            location: read_project_location(&env),
            metadata_url: read_project_metadata_url(&env),
        };
        write_certificate(&env, cert);

        RetirementEvent {
            from: from.clone(),
            amount,
            timestamp,
        }
        .publish(&env);

        CertificateMintedEvent {
            id: cert_id,
            owner: from.clone(),
            amount,
        }
        .publish(&env);

        BurnEvent { from, amount }.publish(&env);

        Ok(())
    }

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount)?;

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        BurnEvent { from, amount }.publish(&env);
        Ok(())
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) -> Result<(), Error> {
        spender.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &spender)?;
        require_not_blacklisted(&env, &from)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&env, from.clone(), spender, amount)?;
        spend_balance(&env, from.clone(), amount)?;

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        BurnEvent { from, amount }.publish(&env);
        Ok(())
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;
        require_not_blacklisted(&env, &spender)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(
            &env,
            from.clone(),
            spender.clone(),
            amount,
            expiration_ledger,
        )?;

        ApproveEvent {
            from,
            spender,
            amount,
            expiration_ledger,
        }
        .publish(&env);
        Ok(())
    }

    // ── Inspection View Functions ─────────────────────────────────────────────

    pub fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, id)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&env, from, spender)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn total_retired(env: Env) -> i128 {
        read_total_retired(&env)
    }

    pub fn name(env: Env) -> String {
        read_name(&env)
    }

    pub fn symbol(env: Env) -> String {
        read_symbol(&env)
    }

    pub fn decimals(env: Env) -> u32 {
        read_decimals(&env)
    }

    pub fn get_certificate(env: Env, id: u32) -> Option<CertificateRecord> {
        read_certificate(&env, id)
    }

    pub fn get_retirement(env: Env, id: u64) -> Option<RetirementRecord> {
        read_retirement(&env, id)
    }

    pub fn certificate_count(env: Env) -> u32 {
        read_next_certificate_id(&env)
    }

    pub fn admin(env: Env) -> Address {
        read_administrator(&env)
    }

    pub fn rbac_contract(env: Env) -> Address {
        crate::storage::read_rbac_contract(&env)
    }

    pub fn is_verifier(env: Env, addr: Address) -> bool {
        is_verifier(&env, &addr)
    }
}
