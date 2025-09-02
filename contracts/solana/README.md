# ZetaChain Solana Integration 🚀

This directory contains ZetaChain's Universal NFT implementation for Solana, enabling cross-chain NFT transfers between Solana and EVM chains through ZetaChain's protocol.

## 🌉 Cross-Chain Architecture

The Solana Universal NFT program integrates with ZetaChain's cross-chain infrastructure:

- **Outbound Transfers**: Burn NFTs on Solana → Send via ZetaChain Gateway → Mint on destination chain
- **Inbound Transfers**: Burn on source chain → ZetaChain processing → Mint on Solana with full metadata
- **Security**: TSS signature verification, replay protection, and comprehensive error handling

## 📦 Contents

- **[Universal NFT Program](./universal-nft/)** - Complete Solana Universal NFT implementation
- **[Architecture Documentation](./universal-nft/ARCHITECTURE_DIAGRAM.md)** - Technical architecture and flow diagrams
- **[Cross-Chain Demo](./universal-nft/CROSS_CHAIN_DEMONSTRATION.md)** - Working cross-chain transfer demonstration

## 🔗 Integration with Protocol Contracts

This implementation is designed to work with [ZetaChain's Protocol Contracts for Solana](https://github.com/zeta-chain/protocol-contracts-solana):

- **Gateway Integration**: CPI calls to ZetaChain gateway program
- **TSS Verification**: Compatible with ZetaChain's threshold signature scheme
- **Message Format**: Structured cross-chain messaging with proper serialization

## 🛠️ Development Requirements

- **Rust**: 1.70+
- **Solana CLI**: 1.18+
- **Anchor Framework**: 0.30+
- **Node.js**: 18+ (for testing and deployment scripts)

## 🚀 Quick Start

### Installation & Setup

```bash
# Clone the repository
git clone https://github.com/zeta-chain/standard-contracts.git
cd standard-contracts/contracts/solana/universal-nft

# Install dependencies
anchor build
npm install

# Run tests
anchor test

# Deploy to devnet
solana config set --url devnet
anchor deploy
```

### Testing Cross-Chain Functionality

```bash
# Run deployment verification
node demo/live-integration-test.js

# Test cross-chain message processing
node test-deployment.js
```

## 🌐 Live Deployment

The Universal NFT Program is deployed and tested on Solana Devnet:

- **Program ID**: `Gc1BJg4sYAYGnKBStAHLTdVRLR3fA7DPc7t9G7vjKa1i`
- **Network**: Solana Devnet
- **Status**: ✅ Deployed & Verified
- **Explorer**: [View on Solana Explorer](https://explorer.solana.com/address/Gc1BJg4sYAYGnKBStAHLTdVRLR3fA7DPc7t9G7vjKa1i?cluster=devnet)

## 📊 Program Instructions

The Universal NFT Program provides the following instructions:

| Instruction | Description | Purpose |
|-------------|-------------|---------|
| `initialize_program` | Initialize program configuration | Setup gateway and collection |
| `mint_nft` | Mint NFT with metadata | Standard NFT creation |
| `burn_for_cross_chain` | Burn NFT for cross-chain transfer | Outbound cross-chain |
| `mint_from_cross_chain` | Mint NFT from cross-chain message | Inbound cross-chain |
| `on_call` | Process gateway messages | Cross-chain message handling |
| `on_revert` | Handle failed transfers | Error recovery |

## 🔐 Security Features

- **TSS Integration**: ECDSA secp256k1 signature verification
- **Replay Protection**: Nonce-based message ordering
- **Authority Control**: Comprehensive access control
- **State Consistency**: Atomic cross-chain state updates
- **Error Recovery**: Complete revert mechanisms

## 🎯 Use Cases

- **Cross-chain Games**: NFT assets that work across multiple chains
- **Universal Marketplaces**: Trade NFTs regardless of origin chain
- **Identity Systems**: Cross-chain identity and reputation tokens
- **Collectibles**: Seamless transfer between gaming ecosystems

## 📚 Documentation

- **[Program Architecture](./universal-nft/ARCHITECTURE_DIAGRAM.md)** - Complete technical architecture
- **[Cross-Chain Demo](./universal-nft/CROSS_CHAIN_DEMONSTRATION.md)** - Working implementation showcase
- **[API Reference](./universal-nft/README.md)** - Detailed program documentation
- **[ZetaChain Docs](https://www.zetachain.com/docs/developers/chains/solana/)** - Official Solana integration docs

## 🤝 Contributing

This implementation follows ZetaChain's Universal NFT standard while addressing Solana-specific requirements:

- **Compute Budget Optimization**: Efficient account structures and minimal CPI calls
- **Rent Exemption**: Automatic account sizing and rent management
- **SPL Token Integration**: Native Solana token standards
- **Metaplex Compatibility**: Full NFT metadata support

For issues, feature requests, or contributions, please refer to the main [ZetaChain Standard Contracts](https://github.com/zeta-chain/standard-contracts) repository.

---

**Bringing Universal NFTs to Solana - Seamless cross-chain NFT experiences** 🌉