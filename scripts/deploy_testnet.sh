#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <SOURCE_SECRET_KEY>"
  echo "Example: $0 SBXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  exit 1
fi

SOURCE_KEY="$1"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "Building WASM artifacts..."
cargo build --target wasm32-unknown-unknown --release -p rbac -p carbon_credit_token -p escrow

declare -A WASMS=(
  [rbac]="target/wasm32-unknown-unknown/release/rbac.wasm"
  [carbon_credit_token]="target/wasm32-unknown-unknown/release/carbon_credit_token.wasm"
  [escrow]="target/wasm32-unknown-unknown/release/escrow.wasm"
)

for name in "${!WASMS[@]}"; do
  path="${WASMS[$name]}"
  if [ ! -f "$path" ]; then
    echo "ERROR: Missing compiled WASM for $name: $path"
    exit 1
  fi
done

function deploy_contract() {
  local name="$1"
  local wasm_path="$2"

  echo
  echo "Deploying $name..."
  stellar contract deploy --network testnet --source "$SOURCE_KEY" --wasm "$wasm_path"
  echo
}

deploy_contract "rbac" "${WASMS[rbac]}"
deploy_contract "carbon_credit_token" "${WASMS[carbon_credit_token]}"
deploy_contract "escrow" "${WASMS[escrow]}"

cat <<'EOF'
Deployment complete.

Next steps:
1) Save the contract IDs returned by each deploy.
2) Initialize RBAC first.
3) Initialize Carbon Credit Token with the RBAC address.
4) Initialize Escrow.

See DEPLOYMENT.md for example initialization commands.
EOF
