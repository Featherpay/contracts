#!/bin/bash
# Unpause TipRouter contract (admin only)
#
# Usage: ./scripts/unpause.sh [network] [contract_id]
#
# Required environment variables:
#   ADMIN_SECRET_KEY - Secret key of admin account (for signing)
#   ADMIN_ADDRESS - Public address of admin account
#   NETWORK_PASSPHRASE - Network passphrase
#   RPC_URL - Soroban RPC endpoint URL
#
# Example:
#   export ADMIN_SECRET_KEY="SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
#   export ADMIN_ADDRESS="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
#   export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
#   export RPC_URL="https://soroban-testnet.stellar.org"
#   ./scripts/unpause.sh testnet CONTRACT_ID

set -euo pipefail

NETWORK="${1:-testnet}"
CONTRACT_ID="${2:-}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [[ -z "$CONTRACT_ID" ]]; then
    echo -e "${RED}Usage: $0 [network] [contract_id]${NC}"
    exit 1
fi

if [[ -z "${ADMIN_SECRET_KEY:-}" ]]; then
    echo -e "${RED}Error: ADMIN_SECRET_KEY environment variable is required${NC}"
    exit 1
fi

if [[ -z "${ADMIN_ADDRESS:-}" ]]; then
    echo -e "${RED}Error: ADMIN_ADDRESS environment variable is required${NC}"
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

echo -e "${GREEN}=== Unpausing TipRouter Contract ===${NC}"
echo "Contract: $CONTRACT_ID"

echo -e "${YELLOW}Invoking unpause...${NC}"
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$ADMIN_ADDRESS" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --secret-key "$ADMIN_SECRET_KEY" \
    -- unpause \
    --admin "$ADMIN_ADDRESS"

echo -e "${GREEN}Contract unpaused successfully!${NC}"
echo "Tip processing has resumed."