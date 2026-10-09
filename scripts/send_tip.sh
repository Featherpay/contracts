#!/bin/bash
# Invoke send_tip on TipRouter contract
#
# Usage: ./scripts/send_tip.sh [network] [contract_id] [tipper] [creator] [amount] [asset]
#
# Required environment variables:
#   TIPPER_SECRET_KEY - Secret key of tipper account (for signing)
#   NETWORK_PASSPHRASE - Network passphrase
#   RPC_URL - Soroban RPC endpoint URL
#
# Example:
#   export TIPPER_SECRET_KEY="SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
#   export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
#   export RPC_URL="https://soroban-testnet.stellar.org"
#   ./scripts/send_tip.sh testnet CONTRACT_ID TIPPER_ADDRESS CREATOR_ADDRESS 1000000 USDC_ADDRESS

set -euo pipefail

NETWORK="${1:-testnet}"
CONTRACT_ID="${2:-}"
TIPPER="${3:-}"
CREATOR="${4:-}"
AMOUNT="${5:-}"
ASSET="${6:-}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [[ -z "$CONTRACT_ID" || -z "$TIPPER" || -z "$CREATOR" || -z "$AMOUNT" || -z "$ASSET" ]]; then
    echo -e "${RED}Usage: $0 [network] [contract_id] [tipper] [creator] [amount] [asset]${NC}"
    echo ""
    echo "Example:"
    echo "  $0 testnet CONTRACT_ID TIPPER_ADDRESS CREATOR_ADDRESS 1000000 USDC_ADDRESS"
    exit 1
fi

if [[ -z "${TIPPER_SECRET_KEY:-}" ]]; then
    echo -e "${RED}Error: TIPPER_SECRET_KEY environment variable is required${NC}"
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

echo -e "${GREEN}=== Sending Tip ===${NC}"
echo "Contract: $CONTRACT_ID"
echo "Tipper: $TIPPER"
echo "Creator: $CREATOR"
echo "Amount: $AMOUNT"
echo "Asset: $ASSET"

# Build and invoke
echo -e "${YELLOW}Invoking send_tip...${NC}"
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$TIPPER" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --secret-key "$TIPPER_SECRET_KEY" \
    -- send_tip \
    --tipper "$TIPPER" \
    --creator "$CREATOR" \
    --amount "$AMOUNT" \
    --asset "$ASSET"

echo -e "${GREEN}Tip sent successfully!${NC}"