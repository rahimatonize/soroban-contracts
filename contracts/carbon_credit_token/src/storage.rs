use soroban_sdk::{contracttype, Address, Bytes, Env};

// ── TTL Constants (standardized across all contracts) ─────────────────────────
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day at 5s/ledger
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days at 5s/ledger

pub const BALANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day at 5s/ledger
pub const BALANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days at 5s/ledger

// ── Allowance Types ────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

// ── Storage Keys ───────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // Admin / roles
    Admin,
    SuperAdmin,
    RbacContract,
    Verifier(Address),
    Blacklisted(Address),

    // Ledger/accounting
    Balance(Address),
    Allowance(AllowanceDataKey),
    TotalSupply,
    TotalRetired,
    UsedReportHash(Bytes),

    // Metadata
    Name,
    Symbol,
    Decimals,

    // Init flag
    Initialized,

    // Project Metadata
    ProjectName,
    Vintage,
    Location,
    MetadataUrl,

    // NFT Data
    NextCertificateID,
    Certificate(u32),

    // Retirement Records
    NextRetirementID,
    Retirement(u64),
}

// ── Initialization ─────────────────────────────────────────────────────────────
pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(e: &Env) {
    e.storage().instance().set(&DataKey::Initialized, &true);
}

// ── RBAC Contract ──────────────────────────────────────────────────────────────
pub fn write_rbac_contract(e: &Env, rbac_id: &Address) {
    e.storage().instance().set(&DataKey::RbacContract, rbac_id);
}

pub fn read_rbac_contract(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::RbacContract)
        .expect("rbac contract address not set")
}

// ── Supply Accounting ──────────────────────────────────────────────────────────
pub fn read_total_supply(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0)
}

pub fn write_total_supply(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalSupply, &amount);
}

pub fn read_total_retired(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalRetired).unwrap_or(0)
}

pub fn write_total_retired(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalRetired, &amount);
}

pub fn is_report_hash_used(e: &Env, hash: &Bytes) -> bool {
    e.storage().instance().has(&DataKey::UsedReportHash(hash.clone()))
}

pub fn mark_report_hash_used(e: &Env, hash: &Bytes) {
    e.storage().instance().set(&DataKey::UsedReportHash(hash.clone()), &true);
}
