use soroban_sdk::{Address, Env};

use crate::storage::DataKey;

// ── Administrator ─────────────────────────────────────────────────────────────

pub fn read_administrator(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("administrator not set")
}

pub fn write_administrator(e: &Env, id: &Address) {
    e.storage().instance().set(&DataKey::Admin, id);
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

// ── Verifier Role ─────────────────────────────────────────────────────────────

pub fn is_verifier(e: &Env, addr: &Address) -> bool {
    e.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Verifier(addr.clone()))
        .unwrap_or(false)
}

pub fn grant_verifier(e: &Env, verifier: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::Verifier(verifier.clone()), &true);
}

pub fn revoke_verifier(e: &Env, verifier: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Verifier(verifier.clone()));
}

// ── Blacklist ─────────────────────────────────────────────────────────────────

pub fn is_blacklisted(e: &Env, addr: &Address) -> bool {
    e.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Blacklisted(addr.clone()))
        .unwrap_or(false)
}

pub fn blacklist_address(e: &Env, addr: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::Blacklisted(addr.clone()), &true);
}

pub fn unblacklist_address(e: &Env, addr: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Blacklisted(addr.clone()));
}
