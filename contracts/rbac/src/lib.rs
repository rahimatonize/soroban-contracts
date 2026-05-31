#![no_std]

mod error;
mod storage;

pub use error::Error;
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

use storage::{is_admin, read_role, read_super_admin, write_admin, write_role, write_super_admin};

#[contract]
pub struct RbacContract;

#[contractimpl]
impl RbacContract {
    /// Initializes the contract with a SuperAdmin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_initialized(&env);
        write_super_admin(&env, &admin);
        write_admin(&env, &admin);
        write_role(&env, &admin, storage::RoleType::SuperAdmin);

        Ok(())
    }

    // --- Role Management ---

    pub fn grant_admin(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        write_admin(&env, &account);
        write_role(&env, &account, storage::RoleType::Admin);

        Ok(())
    }

    pub fn grant_verifier(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        write_role(&env, &account, storage::RoleType::Verifier);

        Ok(())
    }

    pub fn grant_trader(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        write_role(&env, &account, storage::RoleType::Trader);

        Ok(())
    }

    pub fn revoke_role(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }

        if storage::is_super_admin(&env, &account) {
            return Err(Error::CannotRemoveSuperAdmin);
        }

        match read_role(&env, &account) {
            Some(storage::RoleType::Admin) => {
                storage::revoke_admin(&env, &account);
                storage::remove_role(&env, &account);
                Ok(())
            }
            Some(storage::RoleType::Verifier) => {
                storage::revoke_verifier(&env, &account);
                Ok(())
            }
            Some(storage::RoleType::Trader) => {
                storage::revoke_trader(&env, &account);
                Ok(())
            }
            Some(storage::RoleType::SuperAdmin) => Err(Error::CannotRemoveSuperAdmin),
            None => Err(Error::RoleNotAssigned),
        }
    }

    // --- Batch Operations ---

    pub fn assign_roles_batch(env: Env, super_admin: Address, assignments: Vec<(Address, Symbol)>) -> Result<(), Error> {
        super_admin.require_auth();
        if !storage::is_super_admin(&env, &super_admin) {
            return Err(Error::Unauthorized);
        }

        for assignment in assignments.iter() {
            let (account, role_str) = assignment;
            let role_type = match role_str {
                r if r == Symbol::new(&env, "Admin") => storage::RoleType::Admin,
                r if r == Symbol::new(&env, "Verifier") => storage::RoleType::Verifier,
                r if r == Symbol::new(&env, "Trader") => storage::RoleType::Trader,
                _ => return Err(Error::InvalidRole),
            };
            write_role(&env, &account, role_type);
            if role_type == storage::RoleType::Admin {
                write_admin(&env, &account);
            }
        }

        Ok(())
    }

    pub fn revoke_roles_batch(env: Env, super_admin: Address, revocations: Vec<(Address, Symbol)>) -> Result<(), Error> {
        super_admin.require_auth();
        if !storage::is_super_admin(&env, &super_admin) {
            return Err(Error::Unauthorized);
        }

        for revocation in revocations.iter() {
            let (account, role_str) = revocation;
            if storage::is_super_admin(&env, &account) {
                return Err(Error::CannotRemoveSuperAdmin);
            }
            let expected_role = match role_str {
                r if r == Symbol::new(&env, "Admin") => storage::RoleType::Admin,
                r if r == Symbol::new(&env, "Verifier") => storage::RoleType::Verifier,
                r if r == Symbol::new(&env, "Trader") => storage::RoleType::Trader,
                _ => return Err(Error::InvalidRole),
            };
            if let Some(current_role) = read_role(&env, &account) {
                if current_role == expected_role {
                    match current_role {
                        storage::RoleType::Admin => {
                            storage::revoke_admin(&env, &account);
                            storage::remove_role(&env, &account);
                        }
                        storage::RoleType::Verifier => {
                            storage::revoke_verifier(&env, &account);
                        }
                        storage::RoleType::Trader => {
                            storage::revoke_trader(&env, &account);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    // --- Role Checks ---

    pub fn has_role(env: Env, account: Address, role: String) -> bool {
        let role_type = if role == String::from_str(&env, "Admin") {
            storage::RoleType::Admin
        } else if role == String::from_str(&env, "Verifier") {
            storage::RoleType::Verifier
        } else if role == String::from_str(&env, "Trader") {
            storage::RoleType::Trader
        } else if role == String::from_str(&env, "SuperAdmin") {
            storage::RoleType::SuperAdmin
        } else {
            return false;
        };

        if let Some(assigned) = read_role(&env, &account) {
            assigned == role_type
        } else {
            false
        }
    }

    pub fn is_admin(env: Env, account: Address) -> bool {
        is_admin(&env, &account)
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        write_super_admin(&env, &new_admin);
        write_admin(&env, &new_admin);
        write_role(&env, &new_admin, storage::RoleType::SuperAdmin);

        Ok(())
    }
}
