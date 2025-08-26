#!/bin/bash

set -e
set -x
set -o pipefail

echo "🚀 Starting Universal NFT Devnet Testing"
echo "Testing cross-chain flow: Solana Devnet ↔ ZetaChain Testnet ↔ Base Sepolia"

# Compile contracts
echo "📦 Compiling contracts..."
npx hardhat compile --force --quiet

# Deploy on ZetaChain testnet
echo "🌐 Deploying on ZetaChain testnet..."
UNIVERSAL=$(npx hardhat nft:deploy --name ZetaChainUniversalNFT --network zeta_testnet --gateway 0x6c533f7fe93fae114d0954697069df33c9b74fd7 --uniswapRouter 0x2ca7d64A7EFE2D62A725E2B35Cf7230D6677FfEe --gasLimit 500000 --json | jq -r '.contractAddress')
echo "✅ ZetaChain Universal NFT deployed at: $UNIVERSAL"

# Deploy on Base Sepolia
echo "🔵 Deploying on Base Sepolia..."
CONNECTED_BASE=$(npx hardhat nft:deploy --name EVMUniversalNFT --network base_sepolia --gateway 0x0c487a766110c85d301d96e33579c5b317fa4995 --gasLimit 500000 --json | jq -r '.contractAddress')
echo "✅ Base Sepolia Universal NFT deployed at: $CONNECTED_BASE"

# Deploy on BNB testnet
echo "🟡 Deploying on BNB testnet..."
CONNECTED_BNB=$(npx hardhat nft:deploy --name EVMUniversalNFT --network bsc_testnet --gateway 0x0c487a766110c85d301d96e33579c5b317fa4995 --gasLimit 500000 --json | jq -r '.contractAddress')
echo "✅ BNB testnet Universal NFT deployed at: $CONNECTED_BNB"

# ZRC-20 addresses
ZRC20_BASE=0x236b0DE675cC8F46AE186897fCCeFe3370C9eDeD
ZRC20_BNB=0xd97B1de3619ed2c6BEb3860147E30cA8A7dC9891

echo "🔗 Setting up cross-chain connections..."

# Connect Base Sepolia to ZetaChain
echo "🔗 Connecting Base Sepolia to ZetaChain..."
npx hardhat nft:set-universal --network base_sepolia --contract "$CONNECTED_BASE" --universal "$UNIVERSAL" --json

# Connect BNB to ZetaChain
echo "🔗 Connecting BNB to ZetaChain..."
npx hardhat nft:set-universal --network bsc_testnet --contract "$CONNECTED_BNB" --universal "$UNIVERSAL" --json

# Connect ZetaChain to Base Sepolia
echo "🔗 Connecting ZetaChain to Base Sepolia..."
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_BASE" --zrc20 "$ZRC20_BASE" --json

# Connect ZetaChain to BNB
echo "🔗 Connecting ZetaChain to BNB..."
npx hardhat nft:set-connected --network zeta_testnet --contract "$UNIVERSAL" --connected "$CONNECTED_BNB" --zrc20 "$ZRC20_BNB" --json

echo "✅ Cross-chain connections established!"

# Solana Devnet Integration
echo "🟣 Setting up Solana Devnet integration..."

# Deploy Solana program (if not already deployed)
echo "🟣 Deploying Solana Universal NFT program to devnet..."
cd ../../../protocol-contracts-solana
anchor build
anchor deploy --provider.cluster devnet

# Get the deployed program ID
PROGRAM_ID=$(solana address -k target/deploy/universal_nft-keypair.json)
echo "✅ Solana program deployed at: $PROGRAM_ID"

# Return to NFT contracts directory
cd ../contracts/nft

echo "🚀 Devnet testing setup complete!"
echo ""
echo "📋 Test Scenarios to Run:"
echo "1. Mint NFT on Solana devnet → Send to Base Sepolia"
echo "2. Mint NFT on ZetaChain testnet → Send to Solana devnet"
echo "3. Mint NFT on Base Sepolia → Send to Solana devnet"
echo "4. Complete flow: ZetaChain → Base Sepolia → Solana → ZetaChain"
echo ""
echo "🔗 Contract Addresses:"
echo "ZetaChain Universal NFT: $UNIVERSAL"
echo "Base Sepolia Universal NFT: $CONNECTED_BASE"
echo "BNB Universal NFT: $CONNECTED_BNB"
echo "Solana Program: $PROGRAM_ID"
echo ""
echo "✅ Ready for cross-chain testing on devnet!"
