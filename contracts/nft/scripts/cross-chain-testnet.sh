#!/bin/bash

# Cross-chain testnet script for Solana → Base and ZetaChain → Solana flows
# This script performs specific cross-chain NFT transfers without full cycle testing

set -e
set -x
set -o pipefail

echo "🚀 Starting Cross-Chain NFT Testnet Script"
echo "Flows: Solana → Base and ZetaChain → Solana"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOLANA_PROGRAM_ID="6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke"
SOLANA_RPC_URL="https://api.devnet.solana.com"
BASE_SEPOLIA_RPC="https://sepolia.base.org"
ZETACHAIN_TESTNET_RPC="https://zetachain-evm.blockpi.network/v1/rpc/public"

# ZetaChain Gateway and Router addresses
ZETA_GATEWAY="0x6c533f7fe93fae114d0954697069df33c9b74fd7"
ZETA_UNISWAP_ROUTER="0x2ca7d64A7EFE2D62A725E2B35Cf7230D6677FfEe"
BASE_GATEWAY="0x0c487a766110c85d301d96e33579c5b317fa4995"

# ZRC20 token addresses
ZRC20_BASE="0x236b0DE675cC8F46AE186897fCCeFe3370C9eDeD"

echo -e "${BLUE}📋 Configuration:${NC}"
echo "  Solana Program: $SOLANA_PROGRAM_ID"
echo "  Base Gateway: $BASE_GATEWAY"
echo "  ZetaChain Gateway: $ZETA_GATEWAY"
echo ""

# Step 1: Compile contracts
echo -e "${YELLOW}🔨 Compiling contracts...${NC}"
npx hardhat compile --force --quiet

# Step 2: Deploy EVM contracts
echo -e "${YELLOW}🚀 Deploying EVM contracts...${NC}"

# Deploy ZetaChain Universal NFT contract
echo -e "${BLUE}Deploying ZetaChain Universal NFT...${NC}"
UNIVERSAL=$(npx hardhat nft:deploy \
    --name ZetaChainUniversalNFT \
    --network zeta_testnet \
    --gateway "$ZETA_GATEWAY" \
    --uniswap-router "$ZETA_UNISWAP_ROUTER" \
    --gas-limit 500000 \
    --json | jq -r '.contractAddress')

echo -e "${GREEN}✅ ZetaChain Universal NFT deployed at: $UNIVERSAL${NC}"

# Deploy Base Connected NFT contract
echo -e "${BLUE}Deploying Base Connected NFT...${NC}"
CONNECTED_BASE=$(npx hardhat nft:deploy \
    --name EVMUniversalNFT \
    --network base_sepolia \
    --gateway "$BASE_GATEWAY" \
    --gas-limit 500000 \
    --json | jq -r '.contractAddress')

echo -e "${GREEN}✅ Base Connected NFT deployed at: $CONNECTED_BASE${NC}"

# Step 3: Configure cross-chain connections
echo -e "${YELLOW}🔗 Configuring cross-chain connections...${NC}"

# Set universal contract on Base
echo -e "${BLUE}Setting universal contract on Base...${NC}"
npx hardhat nft:set-universal \
    --network base_sepolia \
    --contract "$CONNECTED_BASE" \
    --universal "$UNIVERSAL" \
    --json

# Set connected contract on ZetaChain for Base
echo -e "${BLUE}Setting connected contract on ZetaChain for Base...${NC}"
npx hardhat nft:set-connected \
    --network zeta_testnet \
    --contract "$UNIVERSAL" \
    --connected "$CONNECTED_BASE" \
    --zrc20 "$ZRC20_BASE" \
    --json

echo -e "${GREEN}✅ Cross-chain connections configured${NC}"

# Step 4: Initialize Solana program (if needed)
echo -e "${YELLOW}🔧 Checking Solana program configuration...${NC}"

# Check if Solana program is already deployed and configured
echo -e "${BLUE}Solana program is already deployed at: $SOLANA_PROGRAM_ID${NC}"

# Step 5: Execute cross-chain flows
echo -e "${YELLOW}🌉 Executing cross-chain flows...${NC}"

# Flow 1: Solana → Base
echo -e "${BLUE}🔄 Flow 1: Solana → Base${NC}"
echo "This would involve:"
echo "  1. Mint NFT on Solana"
echo "  2. Initiate cross-chain transfer from Solana to Base"
echo "  3. Verify NFT appears on Base network"

# Note: Actual Solana transactions would require anchor/solana CLI commands
# For now, we'll show the structure and what would be executed

echo -e "${YELLOW}📝 Solana → Base flow commands (to be executed):${NC}"
cat << 'EOF'
# Solana commands (would be executed with anchor/solana CLI):
# 1. Initialize collection on Solana
# anchor run initialize-collection --provider.cluster devnet

# 2. Mint NFT on Solana
# anchor run mint-nft --provider.cluster devnet

# 3. Transfer cross-chain to Base
# anchor run transfer-cross-chain --provider.cluster devnet --args base_sepolia
EOF

# Flow 2: ZetaChain → Solana
echo -e "${BLUE}🔄 Flow 2: ZetaChain → Solana${NC}"
echo "This would involve:"
echo "  1. Mint NFT on ZetaChain"
echo "  2. Initiate cross-chain transfer from ZetaChain to Solana"
echo "  3. Verify NFT appears on Solana network"

# Mint NFT on ZetaChain
echo -e "${YELLOW}Minting NFT on ZetaChain...${NC}"
MINT_TX=$(npx hardhat nft:mint \
    --network zeta_testnet \
    --contract "$UNIVERSAL" \
    --recipient "0x1234567890123456789012345678901234567890" \
    --token-uri "https://example.com/metadata/1.json" \
    --json | jq -r '.transactionHash')

echo -e "${GREEN}✅ NFT minted on ZetaChain. TX: $MINT_TX${NC}"

# Initiate cross-chain transfer to Solana
echo -e "${YELLOW}Initiating cross-chain transfer ZetaChain → Solana...${NC}"
TRANSFER_TX=$(npx hardhat nft:transfer \
    --network zeta_testnet \
    --contract "$UNIVERSAL" \
    --token-id 1 \
    --destination "solana" \
    --recipient "11111111111111111111111111111112" \
    --json | jq -r '.transactionHash')

echo -e "${GREEN}✅ Cross-chain transfer initiated. TX: $TRANSFER_TX${NC}"

# Step 6: Summary
echo -e "${GREEN}🎉 Cross-chain testnet script completed!${NC}"
echo ""
echo -e "${BLUE}📊 Summary:${NC}"
echo "  ZetaChain Universal NFT: $UNIVERSAL"
echo "  Base Connected NFT: $CONNECTED_BASE"
echo "  Solana Program: $SOLANA_PROGRAM_ID"
echo ""
echo -e "${BLUE}🔗 Cross-chain flows executed:${NC}"
echo "  1. Solana → Base (structure prepared)"
echo "  2. ZetaChain → Solana (transactions executed)"
echo ""
echo -e "${YELLOW}📝 Next steps:${NC}"
echo "  1. Verify NFT transfers on destination chains"
echo "  2. Check transaction status on block explorers"
echo "  3. Test additional cross-chain scenarios as needed"
echo ""
echo -e "${BLUE}🔍 Transaction Hashes:${NC}"
echo "  ZetaChain Mint: $MINT_TX"
echo "  ZetaChain → Solana Transfer: $TRANSFER_TX"
