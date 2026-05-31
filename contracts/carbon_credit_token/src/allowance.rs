use soroban_sdk::{Address, Env};

use crate::error::Error;
use crate::storage::{AllowanceDataKey, AllowanceValue, DataKey};

pub fn read_allowance(e: &Env, from: Address, spender: Address) -> i128 {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    if let Some(allowance) = e.storage().temporary().get::<DataKey, AllowanceValue>(&key) {
        if allowance.expiration_ledger < e.ledger().sequence() {
            0
        } else {
            allowance.amount
        }
    } else {
        0
    }
}

pub fn write_allowance(
    e: &Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) -> Result<(), Error> {
    if amount > 0 && expiration_ledger < e.ledger().sequence() {
        return Err(Error::InvalidExpirationLedger);
    }

    let allowance = AllowanceValue {
        amount,
        expiration_ledger,
    };

    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    e.storage().temporary().set(&key, &allowance);

    if amount > 0 {
        let live_for = expiration_ledger.saturating_sub(e.ledger().sequence());
        e.storage().temporary().extend_ttl(&key, live_for, live_for);
    }

    Ok(())
}

pub fn spend_allowance(
    e: &Env,
    from: Address,
    spender: Address,
    amount: i128,
) -> Result<(), Error> {
    let allowance = read_allowance(e, from.clone(), spender.clone());
    if allowance < amount {
        return Err(Error::InsufficientAllowance);
    }
    if amount > 0 {
        write_allowance(e, from, spender, allowance - amount, e.ledger().sequence())?;
    }
    Ok(())
}
