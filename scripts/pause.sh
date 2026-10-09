#!/bin/bash
# Pause TipRouter contract (admin only)
#
# Usage: ./scripts/pause.sh [network] [contract_id]
#
# Required environment variables:
#   ADMIN_SECRET_KEY - Secret key of admin account (for signing)
#   NETWORK_PASSPHRASE - Network passphrase
#   RPC_URL - Soroban RPC endpoint URL
#
# Example:
#   export ADMIN_SECRET_KEY="SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
#   export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
#   export RPC_URL="https://soroban-testnet.stellar.org"
#   ./scripts/pause.sh testnet CONTRACT_ID

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

if [[ -z "${NETWORK_PASSPHRASE:-}" ]]; then
    echo -e "${RED}Error: NETWORK_PASSPHRASE environment variable is required${NC}"
    exit 1
fi

if [[ -z "${RPC_URL:-}" ]]; then
    echo -e "${RED}Error: RPC_URL environment variable is required${NC}"
    exit 1
fi

echo -e "${YELLOW}=== Pausing TipRouter Contract ===${NC}"
echo "Contract: $CONTRACT_ID"
echo -e "${RED}WARNING: This will stop all tip processing!${NC}"

read -p "Are you sure you want to pause the contract? (yes/no): " confirm
if [[ "$confirm" != "yes" ]]; then
    echo "Cancelled."
    exit 0
fi

echo -e "${YELLOW}Invoking pause...${NC}"
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$ADMIN_ADDRESS" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --secret-key "$ADMIN_SECRET_KEY" \
    -- pause \
    --admin "$ADMIN_ADDRESS"

echo -e "${GREEN}Contract paused successfully!${NC}"
echo -e "${YELLOW}All send_tip and send_tip_with_fee calls will now fail with ContractPaused error.${NC}"