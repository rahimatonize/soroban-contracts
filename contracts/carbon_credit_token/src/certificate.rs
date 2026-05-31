use soroban_sdk::{contracttype, Address, Env, String};
use crate::storage::DataKey;

#[derive(Clone, Debug)]
#[contracttype]
pub struct CertificateRecord {
    pub id: u32,
    pub owner: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub project_name: String,
    pub vintage: String,
    pub location: String,
    pub metadata_url: String,
}

pub fn read_next_certificate_id(e: &Env) -> u32 {
    let key = DataKey::NextCertificateID;
    e.storage().instance().get(&key).unwrap_or(0)
}

pub fn increment_next_certificate_id(e: &Env) -> u32 {
    let id = read_next_certificate_id(e) + 1;
    let key = DataKey::NextCertificateID;
    e.storage().instance().set(&key, &id);
    id
}

pub fn write_certificate(e: &Env, cert: CertificateRecord) {
    let key = DataKey::Certificate(cert.id);
    e.storage().persistent().set(&key, &cert);
}

pub fn read_certificate(e: &Env, id: u32) -> Option<CertificateRecord> {
    let key = DataKey::Certificate(id);
    e.storage().persistent().get(&key)
}
