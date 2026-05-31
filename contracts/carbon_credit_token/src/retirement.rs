use soroban_sdk::{contracttype, Address, Env, String};

use crate::storage::DataKey;

#[derive(Clone, Debug)]
#[contracttype]
pub struct RetirementRecord {
    pub id: u64,
    pub retiree: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub reason: String,
    pub beneficiary: String,
}

pub fn read_next_retirement_id(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::NextRetirementID)
        .unwrap_or(0u64)
}

pub fn increment_next_retirement_id(e: &Env) -> u64 {
    let id = read_next_retirement_id(e) + 1;
    e.storage()
        .instance()
        .set(&DataKey::NextRetirementID, &id);
    id
}

pub fn write_retirement(e: &Env, record: RetirementRecord) {
    e.storage()
        .persistent()
        .set(&DataKey::Retirement(record.id), &record);
}

pub fn read_retirement(e: &Env, id: u64) -> Option<RetirementRecord> {
    e.storage()
        .persistent()
        .get(&DataKey::Retirement(id))
}
