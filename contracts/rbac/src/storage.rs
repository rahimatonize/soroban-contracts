use soroban_sdk::{contracttype, Address, Env};

// ── TTL Constants (standardized across all contracts) ────────────────────────
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day at 5s/ledger
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days at 5s/ledger

// ── Role Types ───────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Debug, Eq)]
#[contracttype]
pub enum RoleType {
    SuperAdmin,
    Admin,
    Verifier,
    Trader,
}

// ── Storage Keys ─────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    SuperAdmin,
    Admin(Address), // Map of addresses with Admin role
    Role(Address),
}

// ── Initialization ────────────────────────────────────────────────────────────
pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(e: &Env) {
    e.storage().instance().set(&DataKey::Initialized, &true);
}

// ── SuperAdmin ────────────────────────────────────────────────────────────────
pub fn read_super_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::SuperAdmin)
        .expect("super admin not set")
}

pub fn write_super_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::SuperAdmin, admin);
}

// ── Role Helpers ──────────────────────────────────────────────────────────────
pub fn read_role(e: &Env, address: &Address) -> Option<RoleType> {
    e.storage()
        .persistent()
        .get(&DataKey::Role(address.clone()))
}

pub fn write_role(e: &Env, address: &Address, role: RoleType) {
    e.storage()
        .persistent()
        .set(&DataKey::Role(address.clone()), &role);
}

pub fn remove_role(e: &Env, address: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Role(address.clone()));
}

// ── Role Write Helpers ───────────────────────────────────────────────────────
pub fn write_admin(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::Admin(address.clone()), &true);
}

pub fn revoke_admin(e: &Env, address: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::Admin(address.clone()));
}

pub fn revoke_verifier(e: &Env, address: &Address) {
    remove_role(e, address);
}

pub fn revoke_trader(e: &Env, address: &Address) {
    remove_role(e, address);
}

// ── Role Checks ───────────────────────────────────────────────────────────────
pub fn is_super_admin(e: &Env, address: &Address) -> bool {
    matches!(read_role(e, address), Some(RoleType::SuperAdmin))
}

pub fn is_admin(e: &Env, address: &Address) -> bool {
    matches!(
        read_role(e, address),
        Some(RoleType::SuperAdmin) | Some(RoleType::Admin)
    )
}
