use soroban_sdk::{contracttype, Address, Env, String};

// TTL Constants (standardized across all contracts)
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day at 5s/ledger
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days at 5s/ledger

/// Verifier profile containing public decentralized profile information
#[derive(Clone)]
#[contracttype]
pub struct VerifierProfile {
    pub name: String,           // Entity name
    pub registration_date: u32, // Registration timestamp (ledger number)
    pub jurisdiction: String,   // Geographic jurisdiction
    pub is_active: bool,        // Whether the verifier is active
}

/// Storage keys for Verifier Registry contract
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Boolean flag to check if contract is initialized
    Initialized,
    /// The SuperAdmin address
    SuperAdmin,
    /// Verifier profile by address
    VerifierProfile(Address),
    /// Set of registered verifiers
    Verifiers(Address),
    /// Report hash by farmer address (latest report) - stored as (verifier, hash, ledger)
    ReportByFarmer(Address),
}

/// Check if the contract has been initialized
pub fn is_initialized(e: &Env) -> bool {
    let key = DataKey::Initialized;
    e.storage().instance().has(&key)
}

/// Mark the contract as initialized
pub fn set_initialized(e: &Env) {
    let key = DataKey::Initialized;
    e.storage().instance().set(&key, &true);
}

/// Read the SuperAdmin address
pub fn read_super_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::SuperAdmin).unwrap()
}

/// Write the SuperAdmin address
pub fn write_super_admin(e: &Env, id: &Address) {
    e.storage().instance().set(&DataKey::SuperAdmin, id);
}

/// Read verifier profile
pub fn read_verifier_profile(e: &Env, addr: &Address) -> Option<VerifierProfile> {
    e.storage()
        .instance()
        .get::<DataKey, VerifierProfile>(&DataKey::VerifierProfile(addr.clone()))
}

/// Write verifier profile
pub fn write_verifier_profile(e: &Env, addr: &Address, profile: &VerifierProfile) {
    e.storage()
        .instance()
        .set(&DataKey::VerifierProfile(addr.clone()), profile);
}

/// Check if verifier is registered
pub fn is_verifier_registered(e: &Env, addr: &Address) -> bool {
    e.storage()
        .instance()
        .has(&DataKey::Verifiers(addr.clone()))
}

/// Register a verifier
pub fn register_verifier(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::Verifiers(addr.clone()), &true);
}

/// Unregister a verifier
pub fn unregister_verifier(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::Verifiers(addr.clone()));
}

/// Report data tuple: (verifier_address, metric_hash_string, submission_ledger)
pub type ReportData = (Address, String, u32);

/// Write report for a farmer
pub fn write_report(e: &Env, farmer: &Address, report: &ReportData) {
    e.storage()
        .instance()
        .set(&DataKey::ReportByFarmer(farmer.clone()), report);
}

/// Read report for a farmer
pub fn read_report(e: &Env, farmer: &Address) -> Option<ReportData> {
    e.storage()
        .instance()
        .get::<DataKey, ReportData>(&DataKey::ReportByFarmer(farmer.clone()))
}
