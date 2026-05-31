# Walkthrough: Soulbound NFT Retirement Certificates

We have successfully integrated non-transferable (Soulbound) NFT retirement certificates into the advanced `CarbonCreditToken` contract. This provides users with permanent, verifiable, and project-specific proof of their carbon offsets.

## Key Accomplishments

### 1. Advanced Architecture Integration
The NFT functionality is now fully compliant with the contract's professional patterns:
- **Role-Based Access Control**: Integrated with the external RBAC contract (Verifier roles).
- **Security Check**: respects the global blacklist system.
- **Robust Error Handling**: Uses custom `Error` enums and `Result` types for all operations.

### 2. Rich Metadata NFTs
Every certificate minted during the `retire` process includes persistent project metadata:
- **Project Name**: e.g., "Amazon Reforestation"
- **Vintage**: e.g., "2023"
- **Location**: e.g., "Brazil"
- **Metadata URL**: Link to audit reports or documentation.

### 3. Unified Storage Model
We replaced the primitive bookkeeping system with a globally-indexed NFT system:
- `get_certificate(id)`: Allows anyone to verify a specific certificate's details.
- `certificate_count()`: Provides the total number of offsets issued by the contract.

## Verification Results

### Automated Tests
Successfully ran the expanded test suite:
- `test_initialize`: Verified project metadata storage.
- `test_retire_and_certificate_issuance`: Confirmed that retiring tokens correctly mints an NFT with the expected owner and project data.
- `test_multiple_retirements`: Verified the global ID counter and multi-issue logic.

```bash
running 3 tests
test test::test_initialize ... ok
test test::test_retire_and_certificate_issuance ... ok
test test::test_multiple_retirements ... ok
```

## Reference Files
- [lib.rs](file:///Users/macbookpro/soroban-contracts/contracts/carbon_credit_token/src/lib.rs) - Core contract logic
- [storage.rs](file:///Users/macbookpro/soroban-contracts/contracts/carbon_credit_token/src/storage.rs) - Updated data keys
- [certificate.rs](file:///Users/macbookpro/soroban-contracts/contracts/carbon_credit_token/src/certificate.rs) - NFT data structure
- [test.rs](file:///Users/macbookpro/soroban-contracts/contracts/carbon_credit_token/src/test.rs) - Verification suite
