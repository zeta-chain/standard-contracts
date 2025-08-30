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
UNIVERSAL=$(npx hardhat nft:deploy --name ZetaChainUniversalNFT --network zeta_testnet --gateway 0x6c533f7fe93fae114d0954697069df33c9b74fd7 --uniswap-router 0x2ca7d64A7EFE2D62A725E2B35Cf7230D6677FfEe --json | jq -r '.contractAddress')
echo "ZetaChain Universal NFT: $UNIVERSAL"

echo "🚀 Deploying EVMUniversalNFT on Base Sepolia (nft:deploy)..."
CONNECTED_BASE=$(npx hardhat nft:deploy --name EVMUniversalNFT --network base_sepolia --gateway 0x0c487a766110c85d301d96e33579c5b317fa4995 --json | jq -r '.contractAddress')
echo "Base Sepolia Universal NFT: $CONNECTED_BASE"

echo "🚀 Deploying EVMUniversalNFT on BNB testnet (nft:deploy)..."
CONNECTED_BNB=$(npx hardhat nft:deploy --name EVMUniversalNFT --network bsc_testnet --gateway 0x0c487a766110c85d301d96e33579c5b317fa4995 --json | jq -r '.contractAddress')
echo "BNB testnet Universal NFT: $CONNECTED_BNB"

CONNECTED_SOLANA="0x48727a61636b6973564d4c436a774a6e7531364b766b6b4843735038726964735a36755176667a697952736e" # TODO: SET HEX OF SOLANA_PROGRAM_ID (e.g: HrzackisVMLCjwJnu16KvkkHCsP8ridsZ6uQvfziyRsn)

# ZRC-20 references
ZRC20_BASE=0x236b0DE675cC8F46AE186897fCCeFe3370C9eDeD
ZRC20_BNB=0xd97B1de3619ed2c6BEb3860147E30cA8A7dC9891
ZRC20_SOL=0xADF73ebA3Ebaa7254E859549A44c74eF7cff7501

echo "🔗 Wiring connections (set-connected / set-universal)..."
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_BASE" --zrc20 "$ZRC20_BASE" --json
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_BNB" --zrc20 "$ZRC20_BNB" --json
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_SOLANA" --zrc20 "$ZRC20_SOL" --json

npx hardhat nft:set-universal --network base_sepolia --contract "$CONNECTED_BASE" --universal "$UNIVERSAL" --json
npx hardhat nft:set-universal --network bsc_testnet --contract "$CONNECTED_BNB" --universal "$UNIVERSAL" --json

echo "✅ EVM side ready."

echo ""
echo "🧪 Minting sample NFTs on Base / BNB (nft:mint)…"
NFT_URI=${NFT_URI:-"https://example.com/nft/metadata.json"}

BASE_MINT_JSON=$(npx hardhat nft:mint --network base_sepolia --contract "$CONNECTED_BASE" --token-uri "$NFT_URI" --name EVMUniversalNFT --gas-limit "$GAS_LIMIT" --json)
BASE_TOKEN_ID=$(echo "$BASE_MINT_JSON" | jq -r '.tokenId')
echo "Base minted tokenId: $BASE_TOKEN_ID"

BNB_MINT_JSON=$(npx hardhat nft:mint --network bsc_testnet --contract "$CONNECTED_BNB" --token-uri "$NFT_URI" --name EVMUniversalNFT --gas-limit "$GAS_LIMIT" --json)
BNB_TOKEN_ID=$(echo "$BNB_MINT_JSON" | jq -r '.tokenId')
echo "BNB minted tokenId: $BNB_TOKEN_ID"

echo ""
echo "🌉 Transferring from Base Sepolia → Solana devnet"
BASE_XFER_JSON=$(npx hardhat nft:transfer \
	--network base_sepolia \
	--contract "$CONNECTED_BASE" \
	--token-id "$BASE_TOKEN_ID" \
	--destination "$ZRC20_SOL" \
	--gas-amount 0.005 \
	--json)
echo "$BASE_XFER_JSON" | jq -r '. | "🚀 Successfully transferred NFT to the contract.\n📜 Contract address: \(.contractAddress)\n🖼 NFT Contract address: \(.contractAddress)\n🆔 Token ID: \(.tokenId)\n🔗 Transaction hash: \(.transferTransactionHash)\n⛽ Gas used: \(.gasUsed)"'
BASE_TX_HASH=$(echo "$BASE_XFER_JSON" | jq -r '.transferTransactionHash')
echo "🔎 Inbound CCTX: https://zetachain-athens.blockpi.network/lcd/v1/public/zeta-chain/crosschain/inboundHashToCctxData/$BASE_TX_HASH"
echo "🔎 Base Sepolia tx: https://sepolia.basescan.org/tx/$BASE_TX_HASH"

echo ""
echo "🌉 Transferring from BNB Testnet → Solana devnet"
BNB_XFER_JSON=$(npx hardhat nft:transfer \
	--network bsc_testnet \
	--contract "$CONNECTED_BNB" \
	--token-id "$BNB_TOKEN_ID" \
	--destination "$ZRC20_SOL" \
	--gas-amount 0.005 \
	--json)
echo "$BNB_XFER_JSON" | jq -r '. | "🚀 Successfully transferred NFT to the contract.\n📜 Contract address: \(.contractAddress)\n🖼 NFT Contract address: \(.contractAddress)\n🆔 Token ID: \(.tokenId)\n🔗 Transaction hash: \(.transferTransactionHash)\n⛽ Gas used: \(.gasUsed)"'
BNB_TX_HASH=$(echo "$BNB_XFER_JSON" | jq -r '.transferTransactionHash')
echo "🔎 Inbound CCTX: https://zetachain-athens.blockpi.network/lcd/v1/public/zeta-chain/crosschain/inboundHashToCctxData/$BNB_TX_HASH"
echo "🔎 BNB Testnet tx: https://testnet.bscscan.com/tx/$BNB_TX_HASH"

echo ""
echo "Note: For Solana, the Universal contract encodes the Solana recipient. Ensure your Universal program handles Solana recipient encoding as per your implementation."
