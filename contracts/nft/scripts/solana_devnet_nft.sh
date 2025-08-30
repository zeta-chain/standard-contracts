#!/usr/bin/env bash
# Solana devnet end-to-end flow: deploy program, initialize config, mint NFT, transfer to ZetaChain.
# Prints the Solana tx signature that triggers the ZetaChain cross-chain tx.
set -euo pipefail

# Colors
BLUE='\033[0;34m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'

# Repo paths
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CLI_TS="$ROOT_DIR/contracts/example/SolanaUniversalNFT.ts"

# Load .env if present (non-overriding): variables already exported in the environment take precedence
if [ -f "$ROOT_DIR/.env" ]; then
  echo -e "${BLUE}🔎 Loading .env from $ROOT_DIR/.env (non-overriding)${NC}"
  while IFS= read -r line; do
    # ignore comments and empty lines
    [[ -z "$line" || "$line" =~ ^# ]] && continue
    key="${line%%=*}"; value="${line#*=}"
    # only export if not already set in the environment
    if [ -z "${!key+x}" ]; then
      export "$key=$value"
    fi
  done < "$ROOT_DIR/.env"
fi

# Configuration (defaults can be overridden via environment)
# Required-ish (we provide sensible defaults for convenience)
: "${GATEWAY_PROGRAM:=ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis}"
: "${GATEWAY_PDA:=2f9SLuUNb7TNeM6gzBwT4ZjbL5ZyKzzHg1Ce9yiquEjj}"
: "${ZC_UNIVERSAL_ADDR:=5AE1702fBF1Db5E7238dC8De0dc28e46C3Dbd36A}"
# Destination ZRC-20: '0' to stay on ZetaChain; or 20-byte hex (no 0x) or with 0x prefix
: "${DEST_ZRC20:=0}"
# Final recipient on destination chain (string). Override to your target.
: "${FINAL_RECIPIENT:=0x0000000000000000000000000000000000000000}"

# SOL deposit (in SOL) for ZetaChain gateway deposit_and_call gas; override via env
: "${DEPOSIT_SOL:=0.02}"

# Optional overrides
: "${SOLANA_URL:=https://api.devnet.solana.com}"
: "${METADATA_URI:=https://example.com/nft/metadata.json}"
: "${NFT_NAME:=Universal NFT}"
: "${NFT_SYMBOL:=UNFT}"

# Export so downstream tools (if any) can reference them
export GATEWAY_PROGRAM GATEWAY_PDA ZC_UNIVERSAL_ADDR DEST_ZRC20 FINAL_RECIPIENT
export SOLANA_URL METADATA_URI NFT_NAME NFT_SYMBOL DEPOSIT_SOL

SOLANA_DIR="$ROOT_DIR/contracts/solana"

echo -e "${BLUE}🔧 Effective configuration:${NC}"
echo -e "  GATEWAY_PROGRAM: ${YELLOW}$GATEWAY_PROGRAM${NC}"
echo -e "  GATEWAY_PDA:     ${YELLOW}$GATEWAY_PDA${NC}"
echo -e "  ZC_UNIVERSAL:    ${YELLOW}$ZC_UNIVERSAL_ADDR${NC}"
echo -e "  DEST_ZRC20:      ${YELLOW}$DEST_ZRC20${NC}"
echo -e "  FINAL_RECIPIENT: ${YELLOW}$FINAL_RECIPIENT${NC}"
echo -e "  DEPOSIT_SOL:     ${YELLOW}$DEPOSIT_SOL SOL${NC}"
echo -e "  SOLANA_URL:      ${YELLOW}$SOLANA_URL${NC}"
echo -e "  METADATA_URI:    ${YELLOW}$METADATA_URI${NC}"
echo -e "  NFT_NAME:        ${YELLOW}$NFT_NAME${NC}"
echo -e "  NFT_SYMBOL:      ${YELLOW}$NFT_SYMBOL${NC}"

echo -e "${BLUE}📦 Building Solana program (anchor build)...${NC}"
cd "$SOLANA_DIR"
solana config set --url "$SOLANA_URL" >/dev/null
anchor build
echo -e "${BLUE}🚀 Deploying Solana program to devnet (anchor deploy)...${NC}"
anchor deploy --provider.cluster devnet

echo -e "${BLUE}🧰 Using CLI at: ${CLI_TS}${NC}"
cd "$ROOT_DIR"

# Ensure ts-node is available
if ! npx --yes ts-node --version >/dev/null 2>&1; then
  echo -e "${RED}ts-node is required (dev dependency). Run: npm i -D ts-node typescript${NC}"; exit 1;
fi

CLI_RUN=(npx ts-node "$CLI_TS")

echo -e "${BLUE}⚙️  Initializing program config (initialize)...${NC}"
"${CLI_RUN[@]}" initialize "$GATEWAY_PROGRAM" "$GATEWAY_PDA"

#echo -e "${BLUE}🔁 Ensuring config is up to date (update-config)...${NC}"
#"${CLI_RUN[@]}" update-config "$GATEWAY_PROGRAM" "$GATEWAY_PDA" - -

echo -e "${BLUE}🎨 Minting NFT on Solana devnet (mint)...${NC}"
MINT_OUT=$("${CLI_RUN[@]}" mint "$METADATA_URI" "$NFT_NAME" "$NFT_SYMBOL" | tee /dev/fd/3 3>&1) 3>/dev/null || true
TOKEN_ID_HEX=$(echo "$MINT_OUT" | awk -F'Token ID: ' 'NF>1{print $2}' | tr -d '\r' | tail -n1)
if [[ -z "${TOKEN_ID_HEX}" ]]; then
  echo -e "${RED}Failed to parse Token ID from CLI output. Output was:\n$MINT_OUT${NC}"; exit 1;
fi

echo -e "${YELLOW}🆔 Token ID (hex): ${TOKEN_ID_HEX}${NC}"

echo -e "${BLUE}🌉 Initiating transfer to ZetaChain (deposit_and_call via transfer)...${NC}"
# Capture full output and exit code without aborting the script immediately
set +e
TRANSFER_OUT=$("${CLI_RUN[@]}" transfer "$TOKEN_ID_HEX" "$ZC_UNIVERSAL_ADDR" "$DEST_ZRC20" "$FINAL_RECIPIENT" "$DEPOSIT_SOL" 2>&1)
TRANSFER_RC=$?
set -e

if [[ $TRANSFER_RC -ne 0 ]]; then
  echo -e "${RED}Transfer command failed (exit $TRANSFER_RC). Full output:${NC}\n$TRANSFER_OUT"
  exit $TRANSFER_RC
fi

# Parse the tx signature from successful output
TX_SIG=$(echo "$TRANSFER_OUT" | awk -F'Tx: ' 'NF>1{print $2}' | tr -d '\r' | tail -n1)
if [[ -z "${TX_SIG}" ]]; then
  echo -e "${RED}Failed to capture transfer transaction signature from output:${NC}\n$TRANSFER_OUT"
  exit 1
fi

echo -e "${GREEN}✅ Transfer initiated. Solana devnet tx signature ${TX_SIG}${NC}"