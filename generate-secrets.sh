#!/bin/bash

echo "🔑 Generating GitHub Secrets for Cross-Chain CI Pipeline"
echo "========================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install Solana CLI if not present
install_solana() {
    if ! command_exists solana; then
        echo -e "${YELLOW}📦 Installing Solana CLI...${NC}"
        sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
        export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
        echo -e "${GREEN}✅ Solana CLI installed${NC}"
    else
        echo -e "${GREEN}✅ Solana CLI already installed${NC}"
    fi
}

# Function to install Node.js dependencies if needed
install_deps() {
    if [ -d "contracts/nft" ]; then
        echo -e "${YELLOW}📦 Installing Node.js dependencies...${NC}"
        cd contracts/nft
        npm install --silent
        cd ../..
        echo -e "${GREEN}✅ Dependencies installed${NC}"
    else
        echo -e "${YELLOW}⚠️  contracts/nft directory not found, skipping npm install${NC}"
    fi
}

echo -e "${BLUE}🚀 Step 1: Setting up environment...${NC}"
install_solana
install_deps

echo ""
echo -e "${BLUE}🚀 Step 2: Generating EVM Private Key...${NC}"

# Generate EVM private key using Node.js
if command_exists node; then
    EVM_PRIVATE_KEY=$(node -e "
        const { ethers } = require('ethers');
        const wallet = ethers.Wallet.createRandom();
        console.log(wallet.privateKey);
    " 2>/dev/null)
    
    if [ $? -eq 0 ] && [ ! -z "$EVM_PRIVATE_KEY" ]; then
        echo -e "${GREEN}✅ EVM Private Key generated${NC}"
        echo -e "${YELLOW}📝 EVM_PRIVATE_KEY:${NC}"
        echo "$EVM_PRIVATE_KEY"
    else
        echo -e "${RED}❌ Failed to generate EVM private key${NC}"
        echo -e "${YELLOW}💡 Manual alternative: Use MetaMask → Account → Export Private Key${NC}"
        EVM_PRIVATE_KEY="0x0000000000000000000000000000000000000000000000000000000000000000"
    fi
else
    echo -e "${RED}❌ Node.js not found${NC}"
    echo -e "${YELLOW}💡 Install Node.js or manually generate private key${NC}"
    EVM_PRIVATE_KEY="0x0000000000000000000000000000000000000000000000000000000000000000"
fi

echo ""
echo -e "${BLUE}🚀 Step 3: Generating Solana Keypair...${NC}"

# Generate Solana keypair
SOLANA_KEYPAIR_FILE="/tmp/solana-keypair-$(date +%s).json"
if solana-keygen new --outfile "$SOLANA_KEYPAIR_FILE" --no-bip39-passphrase --silent; then
    echo -e "${GREEN}✅ Solana keypair generated${NC}"
    echo -e "${YELLOW}📝 SOLANA_KEYPAIR:${NC}"
    cat "$SOLANA_KEYPAIR_FILE"
    
    # Get the public key for reference
    SOLANA_PUBKEY=$(solana-keygen pubkey "$SOLANA_KEYPAIR_FILE")
    echo -e "${YELLOW}🔑 Solana Public Key:${NC} $SOLANA_PUBKEY"
else
    echo -e "${RED}❌ Failed to generate Solana keypair${NC}"
    echo -e "${YELLOW}💡 Manual alternative: Run 'solana-keygen new'${NC}"
fi

echo ""
echo -e "${BLUE}🚀 Step 4: RPC URLs...${NC}"

echo -e "${YELLOW}📝 BASE_SEPOLIA_RPC:${NC}"
echo "https://sepolia.base.org"

echo -e "${YELLOW}📝 ZETACHAIN_RPC:${NC}"
echo "https://rpc.ankr.com/zetachain_evm_testnet"

echo -e "${YELLOW}📝 SOLANA_DEVNET_RPC:${NC}"
echo "https://api.devnet.solana.com"

echo ""
echo -e "${BLUE}🚀 Step 5: Funding Instructions...${NC}"

echo -e "${YELLOW}💰 Base Sepolia (EVM):${NC}"
echo "Get testnet ETH from: https://www.coinbase.com/faucets/base-ethereum-sepolia-faucet"

echo -e "${YELLOW}💰 ZetaChain Testnet:${NC}"
echo "Get testnet ZETA from: https://faucet.zetachain.com/"

echo -e "${YELLOW}💰 Solana Devnet:${NC}"
echo "Run: solana airdrop 2 --url devnet"

echo ""
echo -e "${GREEN}🎉 All secrets generated successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Next Steps:${NC}"
echo "1. Go to your GitHub repository"
echo "2. Settings → Secrets and variables → Actions"
echo "3. Add each secret with the values above"
echo "4. Push your PR to trigger the pipeline!"
echo ""
echo -e "${YELLOW}⚠️  Security Notes:${NC}"
echo "- Never commit private keys to your repository"
echo "- Use dedicated testing wallets with minimal funds"
echo "- These are testnet keys - don't use for mainnet"
echo ""

# Clean up temporary files
if [ -f "$SOLANA_KEYPAIR_FILE" ]; then
    rm "$SOLANA_KEYPAIR_FILE"
fi

echo -e "${GREEN}✨ Ready to deploy!${NC}"
