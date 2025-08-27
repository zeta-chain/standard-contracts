#!/bin/bash

set -euo pipefail
set -x

# Print failing line/command on error for faster triage
trap 'ec=$?; echo "❌ Error (exit $ec) at line $LINENO: $BASH_COMMAND" >&2; exit $ec' ERR

echo "🚀 Starting Universal NFT Devnet Testing"
echo "Testing cross-chain flow: Solana Devnet ↔ ZetaChain Testnet ↔ Base Sepolia"

# Preflight tool checks
for cmd in node npx jq; do
  command -v "$cmd" >/dev/null || { echo "Missing required tool: $cmd" >&2; exit 1; }
done

# Retry helper with exponential backoff
retry() {
  local n=0; local try=${1:-3}; shift
  local delay=2
  until "$@"; do
    n=$((n+1))
    if [[ $n -ge $try ]]; then
      echo "Retry failed after $try attempts: $*" >&2
      return 1
    fi
    echo "Attempt $n/$try failed. Retrying in ${delay}s: $*" >&2
    sleep "$delay"
    delay=$((delay * 2))
  done
}

# Environment overrides (can be set by caller)
export HARDHAT_NETWORK=${HARDHAT_NETWORK:-"base-sepolia"}
export SOLANA_CLUSTER=${SOLANA_CLUSTER:-"devnet"}
export ZETA_CHAIN_ID=${ZETA_CHAIN_ID:-"7001"}

# Load secrets (create .env if missing)
if [[ ! -f .env ]]; then
  echo "⚠️  No .env file found. Creating template..."
  cat > .env << EOF
# Add your private keys here
PRIVATE_KEY=your_private_key_here
SOLANA_PRIVATE_KEY=your_solana_private_key_here
EOF
  echo "❌ Please add your private keys to .env and run again"
  exit 1
fi

# Source environment
set -a
source .env
set +a

echo "🔧 Environment:"
echo "  HARDHAT_NETWORK: $HARDHAT_NETWORK"
echo "  SOLANA_CLUSTER: $SOLANA_CLUSTER"
echo "  ZETA_CHAIN_ID: $ZETA_CHAIN_ID"

# Deploy EVM contracts
echo "📦 Deploying EVM Universal NFT contracts..."
retry npx hardhat run tasks/deploy.ts --network "$HARDHAT_NETWORK"

# Deploy Solana program
echo "🔗 Deploying Solana Universal NFT program..."
cd ../../protocol-contracts-solana
retry anchor build
retry anchor deploy --provider.cluster "$SOLANA_CLUSTER"

# Test cross-chain flows
echo "🧪 Testing cross-chain flows..."

# Test 1: Mint on Solana → send to Base Sepolia
echo "  Test 1: Solana → Base Sepolia"
cd ../contracts/nft
retry npx hardhat run tasks/mint.ts --network "$HARDHAT_NETWORK"

# Test 2: Mint on ZetaChain → send to Solana
echo "  Test 2: ZetaChain → Solana"
retry npx hardhat run tasks/transfer.ts --network "$HARDHAT_NETWORK"

echo "✅ Devnet testing completed successfully!"
echo "📊 Check transaction logs above for cross-chain flow verification"
