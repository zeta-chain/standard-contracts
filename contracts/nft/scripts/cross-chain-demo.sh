#!/bin/bash

# Cross-chain testnet DEMO script for Solana → Base and ZetaChain → Solana flows
# This script demonstrates the structure and commands without requiring funded wallets

set -e
set -x
set -o pipefail

echo "🚀 Cross-Chain NFT Testnet DEMO Script"
echo "Flows: Solana → Base and ZetaChain → Solana"
echo "NOTE: This is a demo version showing the structure. For actual deployment, use funded testnet wallets."

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

echo -e "${GREEN}✅ Contracts compiled successfully${NC}"

# Step 2: Show deployment commands (demo mode)
echo -e "${YELLOW}🚀 EVM Contract Deployment Commands:${NC}"

echo -e "${BLUE}ZetaChain Universal NFT deployment command:${NC}"
echo "npx hardhat nft:deploy \\"
echo "    --name ZetaChainUniversalNFT \\"
echo "    --network zeta_testnet \\"
echo "    --gateway $ZETA_GATEWAY \\"
echo "    --uniswap-router $ZETA_UNISWAP_ROUTER \\"
echo "    --gas-limit 500000 \\"
echo "    --json"

echo ""
echo -e "${BLUE}Base Connected NFT deployment command:${NC}"
echo "npx hardhat nft:deploy \\"
echo "    --name EVMUniversalNFT \\"
echo "    --network base_sepolia \\"
echo "    --gateway $BASE_GATEWAY \\"
echo "    --gas-limit 500000 \\"
echo "    --json"

# Mock deployed addresses for demo
UNIVERSAL="0x1234567890123456789012345678901234567890"
CONNECTED_BASE="0x0987654321098765432109876543210987654321"

echo ""
echo -e "${GREEN}✅ Mock deployed addresses:${NC}"
echo "  ZetaChain Universal NFT: $UNIVERSAL"
echo "  Base Connected NFT: $CONNECTED_BASE"

# Step 3: Show configuration commands
echo ""
echo -e "${YELLOW}🔗 Cross-chain Configuration Commands:${NC}"

echo -e "${BLUE}Set universal contract on Base:${NC}"
echo "npx hardhat nft:set-universal \\"
echo "    --network base_sepolia \\"
echo "    --contract $CONNECTED_BASE \\"
echo "    --universal $UNIVERSAL \\"
echo "    --json"

echo ""
echo -e "${BLUE}Set connected contract on ZetaChain for Base:${NC}"
echo "npx hardhat nft:set-connected \\"
echo "    --network zeta_testnet \\"
echo "    --contract $UNIVERSAL \\"
echo "    --connected $CONNECTED_BASE \\"
echo "    --zrc20 $ZRC20_BASE \\"
echo "    --json"

# Step 4: Show Solana operations
echo ""
echo -e "${YELLOW}🔧 Solana Program Operations:${NC}"
echo -e "${BLUE}Solana program is already deployed at: $SOLANA_PROGRAM_ID${NC}"

echo ""
echo -e "${BLUE}Solana operations would include:${NC}"
echo "1. Initialize collection:"
echo "   anchor run initialize-collection --provider.cluster devnet"
echo ""
echo "2. Mint NFT on Solana:"
echo "   anchor run mint-nft --provider.cluster devnet"
echo ""
echo "3. Set up cross-chain connection:"
echo "   anchor run set-connected --provider.cluster devnet"

# Step 5: Show cross-chain flows
echo ""
echo -e "${YELLOW}🌉 Cross-chain Flow Commands:${NC}"

# Flow 1: Solana → Base
echo -e "${BLUE}🔄 Flow 1: Solana → Base${NC}"
echo "Commands that would be executed:"
echo ""
echo "1. Transfer from Solana to Base:"
echo "   anchor run transfer-cross-chain --provider.cluster devnet -- \\"
echo "     --destination-chain 84532 \\"
echo "     --recipient 0x1234567890123456789012345678901234567890"

# Flow 2: ZetaChain → Solana
echo ""
echo -e "${BLUE}🔄 Flow 2: ZetaChain → Solana${NC}"
echo "Commands that would be executed:"
echo ""
echo "1. Mint NFT on ZetaChain:"
echo "npx hardhat nft:mint \\"
echo "    --network zeta_testnet \\"
echo "    --contract $UNIVERSAL \\"
echo "    --recipient 0x1234567890123456789012345678901234567890 \\"
echo "    --token-uri https://example.com/metadata/1.json \\"
echo "    --json"

echo ""
echo "2. Transfer ZetaChain → Solana:"
echo "npx hardhat nft:transfer \\"
echo "    --network zeta_testnet \\"
echo "    --contract $UNIVERSAL \\"
echo "    --token-id 1 \\"
echo "    --destination solana \\"
echo "    --recipient 11111111111111111111111111111112 \\"
echo "    --json"

# Step 6: Summary and next steps
echo ""
echo -e "${GREEN}🎉 Cross-chain testnet demo completed!${NC}"
echo ""
echo -e "${BLUE}📊 Summary:${NC}"
echo "  ✅ Contract compilation successful"
echo "  ✅ Deployment commands prepared"
echo "  ✅ Configuration commands ready"
echo "  ✅ Cross-chain flow structure defined"
echo ""
echo -e "${YELLOW}📝 To run with real deployments:${NC}"
echo "1. Fund your testnet wallets with:"
echo "   - ZetaChain testnet tokens (for ZetaChain operations)"
echo "   - Base Sepolia ETH (for Base operations)"
echo "   - Solana devnet SOL (for Solana operations)"
echo ""
echo "2. Set your private key:"
echo "   export PRIVATE_KEY=your_actual_private_key"
echo ""
echo "3. Run the actual deployment script:"
echo "   ./cross-chain-testnet.sh"
echo ""
echo -e "${BLUE}🔍 Expected Transaction Flow:${NC}"
echo "  Solana → Base: NFT minted on Solana, transferred to Base"
echo "  ZetaChain → Solana: NFT minted on ZetaChain, transferred to Solana"
echo ""
echo -e "${GREEN}✨ Demo completed successfully!${NC}"
