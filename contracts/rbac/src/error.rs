use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The contract has already been initialized.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not the SuperAdmin.
    Unauthorized = 3,
    /// The address is already assigned to this role.
    RoleAlreadyAssigned = 4,
    /// The address does not have this role.
    RoleNotAssigned = 5,
    /// Cannot remove the SuperAdmin role from the SuperAdmin address.
    CannotRemoveSuperAdmin = 6,
    /// Invalid role type provided.
    InvalidRole = 7,
    /// The address already has a different role.
    AddressHasDifferentRole = 8,
}
