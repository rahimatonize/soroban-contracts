use soroban_sdk::{Env, String};

use crate::storage::DataKey;

pub fn read_name(e: &Env) -> String {
    e.storage().instance().get(&DataKey::Name).unwrap()
}

pub fn read_symbol(e: &Env) -> String {
    e.storage().instance().get(&DataKey::Symbol).unwrap()
}

pub fn read_decimals(e: &Env) -> u32 {
    e.storage().instance().get(&DataKey::Decimals).unwrap()
}

pub fn write_metadata(e: &Env, name: String, symbol: String, decimals: u32) {
    e.storage().instance().set(&DataKey::Name, &name);
    e.storage().instance().set(&DataKey::Symbol, &symbol);
    e.storage().instance().set(&DataKey::Decimals, &decimals);
}

pub fn read_project_name(e: &Env) -> String {
    e.storage().instance().get(&DataKey::ProjectName).unwrap()
}

pub fn read_project_vintage(e: &Env) -> String {
    e.storage().instance().get(&DataKey::Vintage).unwrap()
}

pub fn read_project_location(e: &Env) -> String {
    e.storage().instance().get(&DataKey::Location).unwrap()
}

pub fn read_project_metadata_url(e: &Env) -> String {
    e.storage().instance().get(&DataKey::MetadataUrl).unwrap()
}

pub fn write_project_info(e: &Env, name: String, vintage: String, location: String, url: String) {
    e.storage().instance().set(&DataKey::ProjectName, &name);
    e.storage().instance().set(&DataKey::Vintage, &vintage);
    e.storage().instance().set(&DataKey::Location, &location);
    e.storage().instance().set(&DataKey::MetadataUrl, &url);
}
