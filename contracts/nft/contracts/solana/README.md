# Solana Universal NFT

A cross-chain NFT protocol enabling seamless transfers between Solana and EVM-compatible chains via ZetaChain. This implementation provides a complete solution for NFT interoperability with advanced features like NFT Origin tracking, TSS signature verification, and comprehensive monitoring tools.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Compatible-9945FF)](https://solana.com/)
[![ZetaChain](https://img.shields.io/badge/ZetaChain-Integrated-00D4AA)](https://zetachain.com/)
[![Anchor](https://img.shields.io/badge/Anchor-Framework-orange)](https://anchor-lang.com/)

## 🌟 Features

### Core Functionality
- **Cross-Chain NFT Transfers**: Seamless NFT transfers between Solana and EVM-compatible chains
- **Metadata Preservation**: Complete metadata and provenance tracking across chains
- **NFT Origin System**: Advanced tracking system for NFT provenance and ownership history
- **TSS Signature Verification**: Secure threshold signature verification for cross-chain operations
- **Multi-Chain Support**: Compatible with Ethereum, BSC, Polygon, Base, Arbitrum, and Optimism

### Security & Reliability
- **Threshold Signature Scheme (TSS)**: Decentralized signature verification using ECDSA secp256k1
- **Replay Attack Prevention**: Comprehensive nonce-based protection
- **Gateway Integration**: Secure integration with ZetaChain gateway protocol
- **Comprehensive Testing**: Full test suite covering all scenarios and edge cases

### Developer Experience
- **TypeScript SDK**: Complete client library with type safety and error handling
- **Event Monitoring**: Real-time event subscription and monitoring capabilities
- **Health Monitoring**: Production-ready monitoring and alerting system
- **Comprehensive Documentation**: Detailed guides and API references

## 🚀 Quick Start

### Prerequisites

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Install Anchor CLI
npm install -g @coral-xyz/anchor-cli

# Install Node.js dependencies
npm install
```

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd solana-universal-nft

# Install dependencies
npm install

# Build the program
anchor build

# Run tests
anchor test
```

### Basic Usage

```typescript
import { UniversalNftClient, Network } from './sdk/client';
import { Keypair } from '@solana/web3.js';

// Initialize client
const client = await UniversalNftClient.create({
  network: Network.DEVNET,
  commitment: 'confirmed'
}, {
  keypair: Keypair.generate() // Use your wallet keypair
});

// Initialize a collection
const { collection, signature } = await client.initializeCollection(
  "My NFT Collection",
  "MNC",
  "https://example.com/collection.json",
  [/* TSS address bytes */]
);

// Mint an NFT
const mintResult = await client.mintNft(
  collection,
  "My NFT #1",
  "MNC",
  "https://example.com/nft1.json"
);

// Transfer cross-chain to Ethereum
const transferResult = await client.transferCrossChain(
  collection,
  mintResult.mint,
  11155111, // Ethereum Sepolia chain ID
  [/* recipient address bytes */]
);
```

### Network Configuration

```typescript
// Devnet (for development)
const devnetClient = await UniversalNftClient.create({
  network: Network.DEVNET,
  endpoint: "https://api.devnet.solana.com"
});

// Mainnet (for production)
const mainnetClient = await UniversalNftClient.create({
  network: Network.MAINNET,
  endpoint: "https://api.mainnet-beta.solana.com"
});
```

## 🏗️ Architecture

### NFT Origin System

The NFT Origin system tracks the complete provenance of NFTs across all supported chains:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Solana NFT    │───▶│  ZetaChain Hub  │───▶│   EVM Chain     │
│                 │    │                 │    │                 │
│ • Origin Data   │    │ • TSS Signing   │    │ • Metadata      │
│ • Metadata      │    │ • Message Route │    │ • Ownership     │
│ • Ownership     │    │ • Verification  │    │ • Origin Track  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Cross-Chain Message Flow

1. **Initiation**: User initiates transfer on source chain
2. **Gateway Call**: Message sent to ZetaChain gateway
3. **TSS Signing**: ZetaChain validators sign the cross-chain message
4. **Destination Call**: Signed message delivered to destination chain
5. **Verification**: Destination chain verifies TSS signature
6. **Execution**: NFT minted/transferred on destination chain

### Security Model

- **TSS Verification**: All cross-chain operations require valid TSS signatures
- **Nonce Protection**: Sequential nonce system prevents replay attacks
- **Gateway Authorization**: Only authorized gateways can initiate transfers
- **Origin Validation**: NFT origin data ensures provenance integrity

### ZetaChain Integration

The protocol integrates with ZetaChain's omnichain infrastructure:

- **Gateway Program**: `GatewayAddress111111111111111111111111111`
- **TSS Signatures**: ECDSA secp256k1 signature verification
- **Message Format**: Standardized cross-chain message encoding
- **Event Monitoring**: Real-time cross-chain event tracking

## 🛠️ Development

### Building the Program

```bash
# Build the Anchor program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run the test suite
anchor test
```

### Testing

```bash
# Run all tests
npm test

# Run specific test suites
npm run test:lifecycle
npm run test:gateway
npm run test:origin
npm run test:production

# Run with coverage
npm run test:coverage
```

### Local Development Environment

```bash
# Start local Solana validator
solana-test-validator

# Deploy to local cluster
anchor deploy --provider.cluster localnet

# Run tests against local cluster
anchor test --provider.cluster localnet
```

### Contributing Guidelines

1. **Fork the repository** and create a feature branch
2. **Write tests** for new functionality
3. **Follow code style** guidelines (run `npm run lint`)
4. **Update documentation** for API changes
5. **Submit a pull request** with detailed description

## 📦 Deployment

### Network Deployment

#### Devnet Deployment
```bash
# Deploy to devnet
npm run deploy:devnet

# Validate deployment
npm run validate:devnet
```

#### Testnet Deployment
```bash
# Deploy to testnet
npm run deploy:testnet

# Run integration tests
npm run test:integration:testnet
```

#### Mainnet Deployment
```bash
# Deploy to mainnet (requires additional verification)
npm run deploy:mainnet

# Enable production monitoring
npm run monitor:production
```

### Configuration Requirements

- **TSS Address**: ZetaChain TSS signer address (20 bytes)
- **Gateway Program**: ZetaChain gateway program ID
- **Connected Contracts**: EVM contract addresses for each supported chain
- **Monitoring Setup**: Health check and alerting configuration

### Monitoring and Maintenance

The system includes comprehensive monitoring capabilities:

```typescript
import { HealthMonitor } from './monitoring/health-check';

// Initialize monitoring
const monitor = new HealthMonitor({
  programId: PROGRAM_ID,
  checkInterval: 30000,
  alertThresholds: {
    errorRate: 0.05,
    latency: 5000
  }
});

// Start monitoring
await monitor.start();
```

## 📚 API Reference

### Program Instructions

#### `initialize_collection`
Initialize a new Universal NFT collection.

```rust
pub fn initialize_collection(
    ctx: Context<InitializeCollection>,
    name: String,
    symbol: String,
    uri: String,
    tss_address: [u8; 20],
) -> Result<()>
```

#### `mint_nft`
Mint a new NFT with NFT Origin tracking.

```rust
pub fn mint_nft(
    ctx: Context<MintNft>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()>
```

#### `transfer_cross_chain`
Transfer NFT to another blockchain.

```rust
pub fn transfer_cross_chain(
    ctx: Context<TransferCrossChain>,
    destination_chain_id: u64,
    recipient: Vec<u8>,
) -> Result<()>
```

#### `receive_cross_chain`
Receive NFT from another blockchain with TSS verification.

```rust
pub fn receive_cross_chain(
    ctx: Context<ReceiveCrossChain>,
    token_id: u64,
    message_hash: [u8; 32],
    signature: [u8; 64],
    recovery_id: u8,
    message_data: Vec<u8>,
    nonce: u64,
) -> Result<()>
```

### SDK Documentation

#### Client Initialization
```typescript
const client = await UniversalNftClient.create(config, walletConfig);
```

#### Collection Management
```typescript
// Initialize collection
const { collection } = await client.initializeCollection(name, symbol, uri, tssAddress);

// Get collection data
const collectionData = await client.getCollection(collection);

// Get collections by authority
const collections = await client.getCollectionsByAuthority(authority);
```

#### NFT Operations
```typescript
// Mint NFT
const mintResult = await client.mintNft(collection, name, symbol, uri, recipient);

// Transfer cross-chain
const transferResult = await client.transferCrossChain(collection, mint, chainId, recipient);

// Handle incoming transfer
const receiveResult = await client.onCall(collection, sender, sourceChain, message, nonce);
```

#### Event Monitoring
```typescript
// Subscribe to events
const subscription = await client.onNftMinted((event) => {
  console.log('NFT minted:', event);
});

// Unsubscribe
subscription.unsubscribe();
```

### Event Definitions

#### `NftMinted`
```rust
pub struct NftMinted {
    pub collection: Pubkey,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub token_id: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}
```

#### `CrossChainTransfer`
```rust
pub struct CrossChainTransfer {
    pub collection: Pubkey,
    pub mint: Pubkey,
    pub sender: Pubkey,
    pub destination_chain: u64,
    pub recipient: Vec<u8>,
    pub token_id: u64,
}
```

#### `TokenTransferReceived`
```rust
pub struct TokenTransferReceived {
    pub collection: Pubkey,
    pub token_id: u64,
    pub recipient: Pubkey,
    pub uri: String,
    pub original_sender: Vec<u8>,
    pub nonce: u64,
    pub origin_chain: u64,
    pub original_mint: Option<Pubkey>,
    pub is_returning: bool,
}
```

## 🔒 Security

### Security Considerations

1. **TSS Signature Verification**: All cross-chain operations require valid TSS signatures from ZetaChain validators
2. **Replay Attack Prevention**: Nonce-based system prevents transaction replay
3. **Access Control**: Only authorized accounts can perform administrative operations
4. **Input Validation**: Comprehensive validation of all user inputs and cross-chain messages
5. **Gateway Authorization**: Only authorized gateway programs can initiate cross-chain calls

### Audit Information

- **Security Audits**: [Link to audit reports when available]
- **Bug Bounty**: [Link to bug bounty program when available]
- **Security Contact**: [Security contact information]

### Best Practices

#### For Developers
- Always validate TSS signatures before processing cross-chain messages
- Implement proper error handling for all operations
- Use the provided SDK for type safety and error prevention
- Monitor events for real-time system status

#### For Operators
- Regularly update TSS addresses when ZetaChain validators change
- Monitor system health using the provided monitoring tools
- Implement proper backup and disaster recovery procedures
- Keep connected contract addresses updated across all chains

#### For Users
- Verify transaction details before signing
- Use official interfaces and SDKs only
- Monitor cross-chain transfers for completion
- Report any suspicious activity immediately

## 📖 Documentation

### Detailed Guides
- [Deployment Guide](docs/deployment-guide.md) - Complete deployment instructions
- [Integration Examples](docs/integration-examples.md) - Practical integration tutorials
- [API Reference](docs/api-reference.md) - Detailed API documentation
- [Security Guide](docs/security-guide.md) - Security best practices

### Development Resources
- [Testing Guide](docs/testing-guide.md) - Comprehensive testing strategies
- [Monitoring Guide](docs/monitoring-guide.md) - Production monitoring setup
- [Troubleshooting](docs/troubleshooting.md) - Common issues and solutions

## 🤝 Support

### Community
- **Discord**: [Join our Discord community]
- **Telegram**: [Join our Telegram group]
- **Twitter**: [Follow us on Twitter]

### Resources
- **Documentation**: [Full documentation site]
- **GitHub Issues**: [Report bugs and request features]
- **Stack Overflow**: Tag questions with `solana-universal-nft`

### Professional Support
- **Enterprise Support**: [Contact for enterprise support]
- **Custom Development**: [Contact for custom development services]

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Solana Foundation** for the robust blockchain infrastructure
- **ZetaChain** for the omnichain protocol and TSS technology
- **Anchor Framework** for simplifying Solana development
- **Metaplex** for NFT metadata standards
- **Community Contributors** for their valuable feedback and contributions

---

**Built with ❤️ for the cross-chain future**

For more information, visit our [documentation site](docs/) or join our [community](https://discord.gg/solana-universal-nft).