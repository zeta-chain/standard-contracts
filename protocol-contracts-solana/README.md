# Solana Universal NFT Program

A Solana program for creating and managing Universal NFTs that can be transferred across different blockchain networks through ZetaChain Gateway integration.

## 🚀 Features

- **Cross-Chain NFT Support**: Mint and transfer NFTs across different blockchain networks
- **Metaplex Integration**: Full compatibility with Metaplex Token Metadata standard
- **Security-First Design**: PDA-based account creation, replay protection, and gateway authentication
- **ZetaChain Gateway Integration**: Seamless integration with ZetaChain's cross-chain infrastructure
- **Comprehensive Error Handling**: Detailed error codes and validation

## 📋 Prerequisites

- Solana CLI (v1.18.14+)
- Anchor Framework (v0.30.1+)
- Node.js (v18+)
- Rust (v1.70+)

## 🏗️ Architecture

### Program Structure

```
src/
├── lib.rs                 # Main program entry point
├── mint.rs               # NFT minting instruction
├── handle_incoming.rs    # Cross-chain transfer handler
├── on_call.rs           # Gateway entry point
├── utils.rs             # Utility functions and CPI helpers
├── state/               # Account state definitions
│   ├── mod.rs
│   ├── nft_origin.rs    # NFT origin tracking
│   ├── replay.rs        # Replay protection
│   └── gateway.rs       # Gateway configuration
└── ix/                  # Instruction modules (legacy)
    └── mod.rs
```

### Key Components

1. **Mint Instruction** (`mint.rs`)
   - Creates new NFTs on Solana
   - Generates deterministic token IDs
   - Creates Metaplex metadata and master edition
   - Stores NFT origin information

2. **Cross-Chain Handler** (`handle_incoming.rs`)
   - Processes incoming cross-chain transfers
   - Validates gateway authentication
   - Implements replay protection
   - Restores NFTs from other chains

3. **Gateway Integration** (`on_call.rs`)
   - Entry point for ZetaChain Gateway calls
   - Validates gateway configuration
   - Routes to appropriate handlers

4. **State Management**
   - `NftOrigin`: Tracks NFT origin information
   - `ReplayMarker`: Prevents replay attacks
   - `GatewayConfig`: Gateway authentication

## 🔧 Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/zeta-chain/standard-contracts.git
   cd standard-contracts/protocol-contracts-solana
   ```

2. **Install dependencies**
   ```bash
   # Install Rust dependencies
   cargo build

   # Install Node.js dependencies (if using tests)
   npm install
   ```

3. **Configure Anchor**
   ```bash
   # Copy and configure Anchor.toml
   cp Anchor.toml.example Anchor.toml
   ```

## 🚀 Quick Start

### 1. Build the Program
```bash
anchor build
```

### 2. Deploy to Localnet
```bash
# Start local validator
solana-test-validator

# Deploy program
anchor deploy --provider.cluster localnet
```

### 3. Run Tests
```bash
# Run unit tests
anchor test

# Run integration tests
npm run test:integration
```

## 📖 Usage

### Minting a New NFT

```typescript
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "./target/types/universal_nft";

const program = anchor.workspace.UniversalNft as Program<UniversalNft>;

// Mint new NFT
const tx = await program.methods
  .mintNewNft("https://example.com/metadata.json")
  .accounts({
    payer: payer.publicKey,
    recipient: recipient.publicKey,
    mint: mint.publicKey,
    metadata: metadataPda,
    masterEdition: masterEditionPda,
    recipientTokenAccount: recipientTokenAccount,
    nftOrigin: nftOriginPda,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([payer, mint])
  .rpc();
```

### Cross-Chain Transfer

```typescript
// Handle incoming cross-chain transfer
const payload = {
  tokenId: new Uint8Array(32),
  originChainId: new anchor.BN(1),
  originMint: new PublicKey("..."),
  recipient: recipient.publicKey,
  metadataUri: "https://example.com/nft.json",
  nonce: new anchor.BN(1),
};

const serializedPayload = serializePayload(payload);

const tx = await program.methods
  .handleIncoming(serializedPayload)
  .accounts({
    // ... account configuration
  })
  .rpc();
```

## 🔒 Security Features

### 1. PDA-Based Account Creation
- All program-owned accounts use PDAs for deterministic addressing
- Prevents account spoofing and ensures data integrity

### 2. Replay Protection
- Unique nonce-based replay markers
- Prevents duplicate cross-chain transfers
- Secure nonce generation and validation

### 3. Gateway Authentication
- Validates gateway program configuration
- Ensures only authorized gateways can call instructions
- Configurable gateway program IDs

### 4. Input Validation
- Comprehensive payload validation
- Metadata URI length limits
- Account state verification

## 🧪 Testing

### Unit Tests
```bash
# Run all unit tests
anchor test

# Run specific test
anchor test --skip-local-validator test_mint_nft
```

### Integration Tests
```bash
# Run integration tests
npm run test:integration

# Run with local validator
anchor test --skip-local-validator
```

### Manual Testing
```bash
# Test mint instruction
anchor run test-mint

# Test cross-chain transfer
anchor run test-cross-chain
```

## 📊 Monitoring

### Program Logs
```bash
# Monitor program logs
solana logs <PROGRAM_ID>

# Filter specific instructions
solana logs <PROGRAM_ID> | grep "mint_new_nft"
```

### Transaction Monitoring
```bash
# Get transaction details
solana confirm <SIGNATURE>

# View account data
solana account <ACCOUNT_ADDRESS>
```

## 🔗 ZetaChain Integration

### Gateway Configuration
```typescript
const gatewayConfig = {
  programId: "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS",
  gatewayProgram: "ZETA_GATEWAY_PROGRAM_ID",
  // Additional configuration
};
```

### Cross-Chain Message Format
```typescript
interface CrossChainNftPayload {
  token_id: [u8; 32];
  origin_chain_id: u16;
  origin_mint: PublicKey;
  recipient: PublicKey;
  metadata_uri: string;
  nonce: u64;
}
```

## 🚨 Error Codes

| Error Code | Description |
|------------|-------------|
| `MetadataTooLong` | Metadata URI exceeds maximum length |
| `InvalidMetadataPda` | Invalid metadata PDA |
| `InvalidMasterEditionPda` | Invalid master edition PDA |
| `NftOriginPdaMismatch` | NFT origin PDA mismatch |
| `UnauthorizedGateway` | Unauthorized gateway access |
| `InvalidPayload` | Invalid cross-chain payload |
| `ReplayAttack` | Replay attack detected |
| `ReplayPdaMismatch` | Replay marker PDA mismatch |

## 📈 Performance

### Gas Optimization
- Efficient PDA derivation algorithms
- Minimal account creation overhead
- Optimized CPI calls to Metaplex

### Storage Optimization
- Compact data structures
- Efficient serialization
- Minimal account space usage

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

### Development Guidelines
- Follow Rust coding standards
- Add comprehensive tests
- Update documentation
- Ensure security best practices

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [Project Docs](https://docs.zeta.tech)
- **Issues**: [GitHub Issues](https://github.com/zeta-chain/standard-contracts/issues)
- **Community**: [Discord](https://discord.gg/zetachain)
- **Email**: support@zeta.tech

## 🔄 Changelog

### v1.0.0 (Current)
- Initial release
- Cross-chain NFT support
- ZetaChain Gateway integration
- Comprehensive security features
- Full Metaplex compatibility

## 🙏 Acknowledgments

- ZetaChain team for cross-chain infrastructure
- Metaplex for NFT standards
- Solana Labs for the Solana blockchain
- Anchor team for the development framework
