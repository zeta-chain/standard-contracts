# Universal NFT Program - Deployment Guide

## 🎯 Quick Start

This guide will help you deploy and test the Universal NFT Program on Solana devnet.

### Prerequisites

Ensure you have the following installed:
- **Rust 1.70+**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Solana CLI 1.18+**: `sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"`
- **Anchor CLI 0.30+**: `cargo install --git https://github.com/coral-xyz/anchor avm --locked --force`
- **Node.js 18+**: Use nvm or download from nodejs.org
- **Yarn**: `npm install -g yarn`

### Environment Setup

1. **Configure Solana CLI for devnet:**
```bash
solana config set --url devnet
solana-keygen new  # Generate a new wallet if needed
```

2. **Fund your wallet:**
```bash
solana airdrop 2
```

3. **Install project dependencies:**
```bash
cd universal-nft-program
yarn install
```

## 🚀 Deployment Process

### Step 1: Build the Program

```bash
anchor build
```

This will compile the Rust program and generate the TypeScript client bindings.

### Step 2: Deploy to Devnet

```bash
anchor deploy --provider.cluster devnet
```

Note the Program ID from the output - you'll need this for initialization.

### Step 3: Initialize the Program

```bash
yarn deploy:devnet
```

This script will:
- Initialize the Universal NFT Program
- Create the collection NFT
- Set up all necessary PDAs
- Save deployment information to `deployment-info.json`

### Step 4: Update TSS Configuration

After deployment, update the TSS address with ZetaChain's actual TSS address:

```bash
# Edit the TSS address in scripts/deploy.ts
# Then run:
yarn update-tss
```

## 🧪 Testing and Verification

### Run Comprehensive Tests

```bash
anchor test
```

This will run the full test suite including:
- Program initialization
- NFT minting
- Cross-chain operations
- Gateway integration
- Configuration updates
- Error handling

### Demonstrate Cross-Chain Functionality

```bash
yarn demo
```

This will:
1. Mint a demonstration NFT
2. Burn it for cross-chain transfer
3. Display the transaction hashes and state changes
4. Show program statistics

## 📊 Verification Checklist

After deployment, verify the following:

### ✅ Program Deployment
- [ ] Program deployed to devnet successfully
- [ ] Program ID matches in Anchor.toml
- [ ] All instruction handlers are accessible

### ✅ Collection Setup  
- [ ] Collection NFT created with proper metadata
- [ ] Collection mint authority set correctly
- [ ] Master edition created successfully

### ✅ Cross-Chain Integration
- [ ] Gateway program ID configured
- [ ] TSS address set (initially zeros, update with real TSS)
- [ ] Message handlers (`on_call`, `on_revert`) functional

### ✅ NFT Operations
- [ ] NFT minting works with metadata
- [ ] Associated token accounts created properly
- [ ] Burn mechanism functions correctly
- [ ] State tracking accurate

### ✅ Security Features
- [ ] Authority checks enforced
- [ ] Nonce-based replay protection active
- [ ] Signature verification implemented
- [ ] Error handling comprehensive

## 🌐 Cross-Chain Demonstration

To demonstrate actual cross-chain functionality with ZetaChain:

### Prerequisites for Full Demo
1. **ZetaChain Testnet Setup**: Deploy complementary contracts on ZetaChain testnet
2. **Gateway Integration**: Configure with actual ZetaChain gateway program ID
3. **TSS Configuration**: Set real TSS address from ZetaChain validators

### Demo Flow
```typescript
// 1. Mint NFT on Solana
const { mint } = await client.mintNft(user, authority, {
  name: "Cross-Chain Demo NFT",
  symbol: "DEMO", 
  uri: "https://demo.zetachain.com/nft.json"
});

// 2. Burn for cross-chain transfer
const burnTx = await client.burnForCrossChain(
  user,
  mint,
  1, // Ethereum chain ID
  "0x742C4883a7De56b4D90f8F6f1F6c6b8D8b4d4b42"
);

// 3. Verify state updates
const nftState = await client.getNftState(mint);
console.log("Cross-chain locked:", nftState.isCrossChainLocked);
```

## 📝 Transaction Hashes for Submission

After deployment, you'll have several important transaction hashes:

1. **Program Deployment**: Anchor deploy output
2. **Program Initialization**: From `yarn deploy:devnet` 
3. **Demo NFT Mint**: From `yarn demo`
4. **Cross-Chain Burn**: From `yarn demo`

Save these in your submission:
- Copy from terminal output
- Check `deployment-info.json` for structured data
- Verify on Solana Explorer (devnet.solana.com)

## 🔍 Monitoring and Debugging

### Check Program Logs
```bash
solana logs <PROGRAM_ID>
```

### View Account Data
```bash
# Program config
solana account <PROGRAM_CONFIG_PDA>

# NFT state
solana account <NFT_STATE_PDA>
```

### Verify on Block Explorer
Visit [Solana Explorer (Devnet)](https://explorer.solana.com/?cluster=devnet) and search for:
- Program ID
- Transaction hashes
- Account addresses

## 🛠️ Troubleshooting

### Common Issues

**1. Build Failures**
- Ensure Rust toolchain is up to date: `rustup update`
- Clear build cache: `anchor clean`
- Check Anchor version: `anchor --version`

**2. Deployment Issues**
- Verify sufficient SOL balance: `solana balance`
- Check network connectivity to devnet
- Ensure program keypair is funded

**3. Test Failures**
- Run tests with verbose output: `anchor test -- --nocapture`
- Check for account rent exemption issues
- Verify PDA derivations are correct

**4. Gateway Integration**
- Placeholder gateway ID used in demo
- Real integration requires ZetaChain testnet coordination
- TSS signature verification needs actual TSS address

### Support Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Developer Docs](https://docs.solana.com/)
- [ZetaChain Documentation](https://www.zetachain.com/docs/)
- [GitHub Issues](https://github.com/zeta-chain/standard-contracts/issues)

## 📋 Submission Requirements

For the ZetaChain bounty submission, ensure you have:

1. **Code Repository**: All source code with proper documentation
2. **Deployment Info**: Program ID, transaction hashes, account addresses
3. **Test Results**: Comprehensive test suite results
4. **Demo Video/Screenshots**: Showing cross-chain functionality
5. **Documentation**: This guide, README, and code comments
6. **Security Analysis**: Error handling and access control verification

The deployment will create all necessary artifacts for a complete submission to the zeta-chain/standard-contracts repository.