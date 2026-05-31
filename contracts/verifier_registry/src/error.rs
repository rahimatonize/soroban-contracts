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
    /// The verifier is not registered.
    VerifierNotRegistered = 4,
    /// The verifier is already registered.
    VerifierAlreadyRegistered = 5,
    /// Invalid hash provided.
    InvalidHash = 6,
    /// No report found for the given farmer.
    ReportNotFound = 7,
}
