#!/usr/bin/env bash
# ZetaChain testnet + EVM testnets: deploy universal + connected NFTs, wire connections, and show sample cross-chain flows.
set -euo pipefail
set -x

# Optional env overrides
: "${GAS_LIMIT:=500000}"

# Prefer IPv4 DNS resolution to avoid HH502 in some networks
export NODE_OPTIONS="${NODE_OPTIONS:---dns-result-order=ipv4first}"

# Basic retry helper
retry() { local n=0; local try=${1:-3}; shift; until "$@"; do n=$((n+1)); if [[ $n -ge $try ]]; then return 1; fi; sleep 2; done; }

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "📦 Compiling EVM contracts (hardhat compile)..."
if ! retry 3 npx --no-install hardhat compile; then
	echo "Hardhat compile failed (possibly HH502: compiler download)."
	echo "Tips:"
	echo " - Ensure internet access to binaries.soliditylang.org"
	echo " - Try again (transient). We've set IPv4-first DNS to help."
	echo " - If behind a proxy, set HTTPS_PROXY/HTTP_PROXY env vars."
	exit 1
fi

echo "🚀 Deploying Universal NFT on ZetaChain testnet (nft:deploy)..."
UNIVERSAL=$(npx hardhat nft:deploy --name ZetaChainUniversalNFT --network zeta_testnet --gateway 0x6c533f7fe93fae114d0954697069df33c9b74fd7 --uniswapRouter 0x2ca7d64A7EFE2D62A725E2B35Cf7230D6677FfEe --gasLimit "$GAS_LIMIT" --json | jq -r '.contractAddress')
echo "ZetaChain Universal NFT: $UNIVERSAL"

echo "🚀 Deploying EVMUniversalNFT on Base Sepolia (nft:deploy)..."
CONNECTED_BASE=$(npx hardhat nft:deploy --name EVMUniversalNFT --network base_sepolia --gateway 0x0c487a766110c85d301d96e33579c5b317fa4995 --gasLimit "$GAS_LIMIT" --json | jq -r '.contractAddress')
echo "Base Sepolia Universal NFT: $CONNECTED_BASE"

echo "🚀 Deploying EVMUniversalNFT on BNB testnet (nft:deploy)..."
CONNECTED_BNB=$(npx hardhat nft:deploy --name EVMUniversalNFT --network bsc_testnet --gateway 0x0c487a766110c85d301d96e33579c5b317fa4995 --gasLimit "$GAS_LIMIT" --json | jq -r '.contractAddress')
echo "BNB testnet Universal NFT: $CONNECTED_BNB"

# ZRC-20 references (can be paramaterized later)
ZRC20_BASE=0x236b0DE675cC8F46AE186897fCCeFe3370C9eDeD
ZRC20_BNB=0xd97B1de3619ed2c6BEb3860147E30cA8A7dC9891

echo "🔗 Wiring connections (set-universal / set-connected)..."
npx hardhat nft:set-universal --network base_sepolia --contract "$CONNECTED_BASE" --universal "$UNIVERSAL" --json
npx hardhat nft:set-universal --network bsc_testnet --contract "$CONNECTED_BNB" --universal "$UNIVERSAL" --json
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_BASE" --zrc20 "$ZRC20_BASE" --json
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_BNB" --zrc20 "$ZRC20_BNB" --json

echo "✅ EVM side ready."

echo ""
echo "🧪 Minting sample NFTs on Base / BNB (nft:mint)…"
NFT_URI=${NFT_URI:-"https://example.com/nft/metadata.json"}

BASE_MINT_JSON=$(npx hardhat nft:mint --network base_sepolia --contract "$CONNECTED_BASE" --tokenUri "$NFT_URI" --name EVMUniversalNFT --json)
BASE_TOKEN_ID=$(echo "$BASE_MINT_JSON" | jq -r '.tokenId')
echo "Base minted tokenId: $BASE_TOKEN_ID"

BNB_MINT_JSON=$(npx hardhat nft:mint --network bsc_testnet --contract "$CONNECTED_BNB" --tokenUri "$NFT_URI" --name EVMUniversalNFT --json)
BNB_TOKEN_ID=$(echo "$BNB_MINT_JSON" | jq -r '.tokenId')
echo "BNB minted tokenId: $BNB_TOKEN_ID"

echo ""
echo "🌉 Next: Transfer from EVM → ZetaChain → Solana devnet"
echo "You need the ZRC-20 address of the Solana destination gas token on ZetaChain (export ZRC20_SOLANA)."
echo "Example commands:"
echo "  # Base → ZetaChain → Solana"
echo "  ZRC20_SOLANA=0xYourZRC20Solana npx hardhat nft:transfer \\
	--network base_sepolia \\
	--contract \"$CONNECTED_BASE\" \\
	--tokenId \"$BASE_TOKEN_ID\" \\
	--destination \"$ZRC20_SOLANA\" \\
	--gasAmount 0.1 \\
	--json"
echo ""
echo "  # BNB → ZetaChain → Solana"
echo "  ZRC20_SOLANA=0xYourZRC20Solana npx hardhat nft:transfer \\
	--network bsc_testnet \\
	--contract \"$CONNECTED_BNB\" \\
	--tokenId \"$BNB_TOKEN_ID\" \\
	--destination \"$ZRC20_SOLANA\" \\
	--gasAmount 0.1 \\
	--json"
echo ""
echo "Note: The current transfer task uses an EVM address-type receiver. For Solana, the ZetaChain universal contract must encode the Solana recipient in its message. If your contract requires a 32-byte Solana recipient, use a dedicated task or adjust the contract call accordingly."
