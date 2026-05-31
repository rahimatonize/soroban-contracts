# Deployment Guide

This repository contains three interdependent Soroban contracts:

1. `rbac` - role-based access control contract
2. `carbon_credit_token` - carbon credit token contract
3. `escrow` - escrow marketplace contract

## Deployment Order

The correct deployment order is:

1. `rbac`
2. `carbon_credit_token`
3. `escrow`

This order is required because the carbon credit token contract stores the RBAC contract address during initialization, and the escrow contract interacts with deployed token contracts.

## Initialization Sequence

### 1) Initialize `rbac`

`rbac` must be initialized first with a super-admin account.

Example:

```bash
stellar contract invoke --network testnet --source <SOURCE_SECRET_KEY> \
  --id <RBAC_CONTRACT_ID> --fn initialize --arg <ADMIN_ADDRESS>
```

- `ADMIN_ADDRESS` should be the account that becomes the contract SuperAdmin and Admin.

### 2) Initialize `carbon_credit_token`

`carbon_credit_token` requires:

- `admin`: the same account that will manage the token contract
- `rbac_contract`: the deployed RBAC contract address
- `name`: token name, for example `"Carbon Credit Token"`
- `symbol`: token symbol, for example `"CCT"`
- `decimals`: token decimals, typically `0`

Example:

```bash
stellar contract invoke --network testnet --source <SOURCE_SECRET_KEY> \
  --id <CARBON_TOKEN_CONTRACT_ID> --fn initialize \
  --arg <ADMIN_ADDRESS> \
  --arg <RBAC_CONTRACT_ID> \
  --arg "Carbon Credit Token" \
  --arg "CCT" \
  --arg 0
```

### 3) Initialize `escrow`

`escrow` has a no-argument initialization function.

Example:

```bash
stellar contract invoke --network testnet --source <SOURCE_SECRET_KEY> \
  --id <ESCROW_CONTRACT_ID> --fn initialize
```

## Dependency Graph

- `rbac` is independent and must be deployed first.
- `carbon_credit_token` depends on the `rbac` contract address.
- `escrow` depends on token contracts when creating offers, but does not take token contract addresses during initialization.

When creating an offer in `escrow`, you pass the token addresses for the carbon token and USDC token directly to `create_offer`.

## Testnet Addresses

This repository does not include published testnet contract addresses.

To deploy and capture your own addresses, use the included deployment script below.

## Script for Reproducible Testnet Deploys

Use the provided script at `scripts/deploy_testnet.sh`.

Example:

```bash
chmod +x scripts/deploy_testnet.sh
./scripts/deploy_testnet.sh <SOURCE_SECRET_KEY>
```

This will:

1. Build `rbac`, `carbon_credit_token`, and `escrow` as WASM artifacts.
2. Deploy each contract to the Soroban testnet.
3. Print instructions for the initialization sequence.

## Example Initialization Transaction Sequence

1. Deploy `rbac`, note the returned contract ID.
2. Deploy `carbon_credit_token`, note the returned contract ID.
3. Deploy `escrow`, note the returned contract ID.
4. Initialize `rbac`:

```bash
stellar contract invoke --network testnet --source <SOURCE_SECRET_KEY> \
  --id <RBAC_CONTRACT_ID> --fn initialize --arg <ADMIN_ADDRESS>
```

5. Initialize `carbon_credit_token`:

```bash
stellar contract invoke --network testnet --source <SOURCE_SECRET_KEY> \
  --id <CARBON_TOKEN_CONTRACT_ID> --fn initialize \
  --arg <ADMIN_ADDRESS> \
  --arg <RBAC_CONTRACT_ID> \
  --arg "Carbon Credit Token" \
  --arg "CCT" \
  --arg 0
```

6. Initialize `escrow`:

```bash
stellar contract invoke --network testnet --source <SOURCE_SECRET_KEY> \
  --id <ESCROW_CONTRACT_ID> --fn initialize
```

## Notes

- `carbon_credit_token` stores the RBAC contract address during initialization and uses RBAC for verifier access checks.
- `escrow` only needs token contract addresses when creating offers, not during initialization.
- If you do not already have a funded testnet account, use `stellar keys generate --network testnet --fund`.
