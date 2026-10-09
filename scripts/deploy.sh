#!/bin/bash
# Deploy TipRouter contract to Stellar network
#
# Usage: ./scripts/deploy.sh [network] [admin_address]
#
# Required environment variables:
#   ADMIN_SECRET_KEY - Secret key of admin account (for signing)
#   NETWORK_PASSPHRASE - Network passphrase (e.g., "Test SDF Network ; September 2015" or "Public Global Stellar Network ; September 2015")
#   RPC_URL - Soroban RPC endpoint URL
#
# Example:
#   export ADMIN_SECRET_KEY="SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
#   export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
#   export RPC_URL="https://soroban-testnet.stellar.org"
#   ./scripts/deploy.sh testnet

set -euo pipefail

NETWORK="${1:-testnet}"
ADMIN_ADDRESS="${2:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== TipRouter Contract Deployment ===${NC}"
echo "Network: $NETWORK"

# Check required environment variables
if [[ -z "${ADMIN_SECRET_KEY:-}" ]]; then
    echo -e "${RED}Error: ADMIN_SECRET_KEY environment variable is required${NC}"
    exit 1
fi

if [[ -z "${NETWORK_PASSPHRASE:-}" ]]; then
    echo -e "${RED}Error: NETWORK_PASSPHRASE environment variable is required${NC}"
    exit 1
fi

if [[ -z "${RPC_URL:-}" ]]; then
    echo -e "${RED}Error: RPC_URL environment variable is required${NC}"
    exit 1
fi

# Build contract
echo -e "${YELLOW}Building contract...${NC}"
stellar contract build

WASM_PATH="target/wasm32v1-none/release/tip_router.wasm"
WASM_HASH=$(stellar contract install --wasm "$WASM_PATH" --network "$NETWORK" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE")
echo "WASM Hash: $WASM_HASH"

# Deploy contract
if [[ -n "$ADMIN_ADDRESS" ]]; then
    echo -e "${YELLOW}Deploying contract with admin: $ADMIN_ADDRESS${NC}"
    CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$WASM_HASH" \
        --source "$ADMIN_ADDRESS" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        --secret-key "$ADMIN_SECRET_KEY")
else
    echo -e "${YELLOW}Deploying contract (admin will be set to deployer)${NC}"
    CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$WASM_HASH" \
        --network "$NETWORK" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE" \
        --secret-key "$ADMIN_SECRET_KEY")
fi

echo -e "${GREEN}Contract deployed successfully!${NC}"
echo "Contract ID: $CONTRACT_ID"
echo ""
echo "Add to your .env:"
echo "TIP_ROUTER_CONTRACT_ID=$CONTRACT_ID"

# Verify deployment
echo -e "${YELLOW}Verifying deployment...${NC}"
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- get_fee_bps

echo -e "${GREEN}Deployment verification complete!${NC}"