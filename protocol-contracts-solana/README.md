# Solana Universal NFT Program

A complete implementation of the Universal NFT protocol on Solana, enabling seamless cross-chain NFT transfers between ZetaChain, EVM chains, and Solana.

## 🎯 Overview

This program implements the Universal NFT standard on Solana, providing:

- **Cross-chain NFT minting** with metadata preservation
- **Burn-and-mint transfer mechanism** (no escrow)
- **Collection support** - each new Universal NFT creates a separate collection
- **Replay protection** for cross-chain messages
- **Full Metaplex integration** for metadata and master editions
- **ZetaChain gateway compatibility** for universal messaging

## 🏗️ Architecture

### Core Components

1. **Mint Instruction** (`mint_new_nft`)
   - Creates new SPL token mint
   - Generates unique token ID using SHA256 hash
   - Creates Metaplex metadata and master edition
   - Stores origin information in `nft_origin` PDA
   - Each NFT becomes its own collection

2. **Burn Instruction** (`burn_for_transfer`)
   - Burns NFT for cross-chain transfer
   - Creates replay protection via `replay_marker` PDA
   - Prepares cross-chain message payload
   - Emits transfer events

3. **Handle Incoming** (`handle_incoming`)
   - Processes cross-chain messages from other chains
   - Mints new NFTs with preserved metadata
   - Links to existing origin or creates new origin PDA
   - Maintains cross-chain provenance

### State Management

- **`nft_origin` PDA**: Stores origin chain, token ID, mint, and metadata URI
- **`replay_marker` PDA**: Prevents duplicate cross-chain message processing
- **Token IDs**: Unique across all chains using SHA256 hash of mint + slot + timestamp

## 🚀 Quick Start

### Prerequisites

- Solana CLI (latest stable)
- Anchor CLI (`cargo install --git https://github.com/coral-xyz/anchor avm --locked`)
- Node.js 18+

### Installation

```bash
# Clone the repository
git clone https://github.com/zeta-chain/standard-contracts.git
cd standard-contracts/protocol-contracts-solana

# Install dependencies
npm install

# Build the program
anchor build
```

### Testing

```bash
# Run unit tests
anchor test

# Run specific test
anchor test --skip-local-validator
```

## 🔗 Cross-chain Integration

### Devnet Testing

**⚠️ Important**: Due to potential Solana compatibility issues with the current version of ZetaChain localnet, **devnet testing is required** for Solana integration. Localnet testing is not required at this time.

Run the devnet script to provision EVM contracts and deploy the Solana program:

```bash
cd ../../contracts/nft/scripts
chmod +x devnet.sh
./devnet.sh
```

This demonstrates the flows:
1. Mint on Solana devnet → send to Base Sepolia
2. Mint on ZetaChain testnet → send to Solana devnet
3. Mint on Base Sepolia → send to Solana devnet
4. ZetaChain → Base Sepolia → Solana → ZetaChain

### Gateway Integration

The program is designed to work with ZetaChain's universal messaging protocol:

- **Outbound**: NFTs burned and cross-chain messages sent via gateway
- **Inbound**: Cross-chain messages processed to mint new NFTs
- **Verification**: Gateway signer verification for message authenticity

## 📋 Instructions

### Mint New NFT

```typescript
await program.methods
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

### Burn for Transfer

```typescript
await program.methods
  .burnForTransfer(new anchor.BN(nonce))
  .accounts({
    owner: owner.publicKey,
    mint: mint.publicKey,
    ownerTokenAccount: ownerTokenAccount,
    nftOrigin: nftOriginPda,
    replayMarker: replayMarkerPda,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .signers([owner])
  .rpc();
```

### Handle Incoming Message

```typescript
await program.methods
  .handleIncoming(payloadBytes)
  .accounts({
    payer: payer.publicKey,
    recipient: recipient.publicKey,
    mint: newMint.publicKey,
    metadata: newMetadataPda,
    masterEdition: newMasterEditionPda,
    recipientTokenAccount: newRecipientTokenAccount,
    nftOrigin: newNftOriginPda,
    gatewaySigner: gatewaySigner.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([payer, newMint])
  .rpc();
```

## 🔒 Security Features

- **Replay Protection**: Nonce-based protection against duplicate messages
- **Origin Verification**: PDA-based verification of NFT origins
- **Access Control**: Proper signer validation and ownership checks
- **Gateway Verification**: Integration with ZetaChain's trusted gateway system

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
anchor test

# Run specific test file
anchor test tests/universal_nft.ts
```

### Integration Tests

The test suite covers:
- NFT minting with collection creation
- Cross-chain burn operations
- Incoming message processing
- Replay protection
- Collection separation verification

### Localnet Integration

```bash
# Start ZetaChain localnet
yarn zetachain localnet start

# Run Solana integration
./localnet_solana.sh
```

## 📚 Documentation

- **Architecture**: See `docs/solana_universal_nft_architecture.md`
- **Security**: See `docs/security.md`
- **API Reference**: Generated via `anchor build`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details

## 🔗 Links

- [ZetaChain Documentation](https://www.zetachain.com/docs/)
- [Universal NFT Standard](https://www.zetachain.com/docs/developers/standards/nft/)
- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/)

## 🆘 Support

For technical support and questions:
- Join the [ZetaChain Discord](https://discord.gg/zetachain)
- Open an issue on GitHub
- Check the documentation and examples
