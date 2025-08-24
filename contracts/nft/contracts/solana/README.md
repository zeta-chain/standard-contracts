# Solana Universal NFT Program

## Overview

The Solana Universal NFT program enables cross-chain NFT transfers between Solana and other blockchains through ZetaChain. This program implements the Universal NFT standard, allowing NFTs to move seamlessly across different chains while maintaining their metadata and ownership.

**Program ID**: `GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip`
**Network**: Solana Devnet
**Status**: Deployed and Operational

The program provides full bidirectional NFT transfer capabilities, supporting minting on Solana and transferring to Ethereum, Base Sepolia, BNB Chain, and other ZetaChain-connected networks.

## Core Functions

The Universal NFT program implements four primary functions that enable cross-chain NFT operations:

### Program Configuration

The program utilizes Anchor framework imports for Solana program development, including `anchor_lang::prelude::*` for core functionality and `anchor_spl` modules for token program interactions.

**ZetaChain Gateway Integration**: The program interfaces with the ZetaChain gateway at program ID `ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis` for cross-chain message handling.

### initialize_collection

Creates a unique NFT collection using Program Derived Addresses (PDAs). The function establishes a collection account that stores the program authority, collection mint reference, and a nonce for replay protection. This collection serves as the parent structure for all NFTs minted through the program.

### mint_nft

Mints new NFTs within the established collection. The function handles PDA seed generation and signer management for token operations. Unlike EVM contracts that directly call other contracts, Solana requires explicit authority verification through PDA signing using `CpiContext::new_with_signer`. This ensures only the program can mint NFTs in its collection.

### transfer_cross_chain

Initiates cross-chain NFT transfers by burning the NFT on Solana and emitting a cross-chain message. The function performs a token burn operation that requires precise account configuration, then invokes the ZetaChain gateway to propagate the transfer message to the destination chain. The gateway handles the cross-chain communication protocol to ensure the NFT is recreated on the target blockchain.

### on_call

Processes incoming NFT transfers from other chains. This gateway-only function performs TSS (Threshold Signature Scheme) signature verification to ensure message authenticity. The function decodes cross-chain messages using Borsh serialization (distinct from EVM's ABI encoding) and mints the corresponding NFT to the specified recipient. Only the authorized ZetaChain gateway can invoke this function, preventing unauthorized NFT creation.

## Technical Architecture

### Compute Budget Management

Solana transactions operate within a compute unit limit of 200,000 units. The program is optimized to consume approximately 23,819 compute units per operation, ensuring efficient execution without exceeding transaction limits. Complex operations are structured to minimize computational overhead.

### Rent-Exempt Accounts

Solana accounts require rent payments to maintain state persistence. Each NFT account must be rent-exempt, requiring approximately 0.00203 SOL per account. This ensures permanent storage without risk of account deletion due to insufficient funds.

### Account Model

Solana's account model requires explicit specification of all accounts involved in each instruction. Each program function defines its required account context, including collection accounts, mint accounts, token accounts, and system programs. Proper account configuration is essential for successful transaction execution.

### Program Derived Addresses (PDAs)

PDAs enable programs to control accounts deterministically. The program generates PDAs using specific seeds ("collection" + program_id) to create addresses that only the program can sign for. This mechanism provides secure, programmatic control over NFT collections and associated accounts.

## Cross-Chain Integration

### Transfer Flow

Outgoing transfers from Solana follow this sequence:
1. User initiates transfer specifying destination chain and recipient
2. Program burns the NFT on Solana
3. Program emits a TokenTransfer event containing NFT metadata
4. ZetaChain gateway monitors and captures the event
5. Gateway invokes the Universal NFT contract on the destination chain
6. New NFT is minted on the destination chain with identical metadata

Incoming transfers utilize the `on_call` function for receiving NFTs from other chains.

### Gateway Security

The program implements strict gateway verification to ensure only the authorized ZetaChain gateway (`ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis`) can invoke the `on_call` function. This prevents unauthorized NFT minting and maintains collection integrity.

### Event System

While Solana lacks native events like Ethereum, the Anchor framework's `emit!` macro generates program logs that off-chain services can monitor. These events include TokenMinted, TokenTransfer, and TokenTransferReceived, containing all necessary data for cross-chain NFT reconstruction.

## Testing Framework

### Test Environment

Solana program testing utilizes `solana-test-validator` for local blockchain simulation and `anchor test` for comprehensive testing. The `cargo test-sbf` command executes Solana BPF tests for program validation. Testing requires careful configuration of accounts and proper transaction simulation.

### Deployment

The program deployment to Solana devnet requires approximately 2.43 SOL to cover the 348,920 byte program size. The deployed program ID `GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip` serves as the permanent identifier for all program interactions.

### Ecosystem Integration

The Solana Universal NFT program integrates with a broader cross-chain ecosystem. The `localnet.sh` script orchestrates comprehensive testing across ZetaChain, Ethereum, BNB Chain, and Solana, demonstrating full interoperability between all connected networks.

## Installation Requirements

### Prerequisites
- Solana CLI (v1.18.26 or higher)
- Anchor Framework (v0.29.0 specifically)
- Node.js (v16 or higher)
- Rust (v1.75 or higher)
- Sufficient SOL for deployment and testing

### Installation
```bash
# Navigate to the Solana contracts directory
cd contracts/nft/contracts/solana

# Install Node.js dependencies
npm install

# Build the Anchor program
anchor build
```

### Running Tests
```bash
# Execute BPF tests
cargo test-sbf

# Run complete test suite
anchor test

# Run TypeScript tests
npx ts-node scripts/local-solana.ts

# Run JavaScript tests
node scripts/test-direct.js
```

### Deployment Process
```bash
# Configure Solana CLI for devnet
solana config set --url https://api.devnet.solana.com

# Request devnet SOL (minimum 3 SOL recommended)
solana airdrop 2

# Deploy the program to devnet
anchor deploy --provider.cluster devnet
```

### Usage Examples
```bash
# Initialize NFT collection
node scripts/test-direct.js

# Test Universal NFT functionality
node scripts/test-universal-nft.js

# Test complete functionality
npx ts-node scripts/local-solana.ts
```

## Operational Workflow

### NFT Minting Process
1. Initialize collection through `initialize_collection` function
2. Collection account is created with program authority
3. Execute `mint_nft` to create individual NFTs
4. Each NFT receives dedicated mint and token accounts
5. Program emits `TokenMinted` event for tracking

### Cross-Chain Transfer Process
1. Invoke `transfer_cross_chain` with destination parameters
2. NFT is burned on Solana
3. `TokenTransfer` event is emitted with metadata
4. ZetaChain gateway captures and processes the event
5. NFT is minted on the destination chain

### Cross-Chain Reception Process
1. ZetaChain gateway invokes `on_call` function
2. Program verifies TSS signatures for authenticity
3. Cross-chain message is decoded
4. New NFT is minted for the specified recipient
5. `TokenTransferReceived` event confirms completion

## Ecosystem Architecture

### Supported Networks
The Universal NFT ecosystem encompasses:
- Solana (Program ID: `GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip`)
- Ethereum Sepolia
- Base Sepolia (Chain ID: 84532)
- BNB Testnet
- ZetaChain as the cross-chain messaging layer

Transfer Path: ZetaChain → Ethereum → BNB → Solana → ZetaChain

### Security Model
The program implements TSS (Threshold Signature Scheme) verification to ensure only authenticated messages from ZetaChain can mint NFTs through the `on_call` function. This cryptographic verification prevents unauthorized NFT creation and maintains collection integrity.

### Event Architecture
The program utilizes Anchor's event system to emit structured logs that off-chain services monitor. Three primary events facilitate cross-chain coordination:
- `TokenMinted`: Confirms NFT creation
- `TokenTransfer`: Initiates cross-chain transfer
- `TokenTransferReceived`: Acknowledges incoming NFT

### Integration Testing
The `localnet.sh` script provides comprehensive end-to-end testing across all supported chains, validating the complete NFT lifecycle from creation through multi-chain transfers.

## Program Metrics

### Resource Requirements
- **Program Size**: 348,920 bytes
- **Deployment Cost**: Approximately 2.43 SOL
- **Compute Usage**: 23,819 units per operation
- **Rent per Account**: 0.00203 SOL
- **Transaction Fee**: Approximately 0.000005 SOL

### Technical Details
- **Anchor Version**: 0.29.0 (had to downgrade from 0.31.1 due to compatibility issues)
- **Solana Version**: 1.18.26
- **Deployment Slot**: 402689238 (I'll never forget this number)
- **Program ID**: `GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip`

## Summary

The Solana Universal NFT program provides a robust foundation for cross-chain NFT operations. Through integration with ZetaChain's messaging protocol, the program enables seamless NFT transfers between Solana and EVM-compatible chains while maintaining metadata integrity and ownership verification.

## Account Structure

### Collection Account
```rust
pub struct Collection {
    pub authority: Pubkey,      // Program authority
    pub collection_mint: Pubkey, // Collection NFT mint
    pub nonce: u64,             // Replay protection
}
```

### Required Accounts for Each Instruction

| Instruction | Required Accounts |
|-------------|------------------|
| **initializeCollection** | collection, collectionMint, payer, rent, systemProgram, tokenProgram, metadataProgram |
| **mintNft** | collection, nftMint, nftTokenAccount, recipient, nftMetadata, payer, rent, systemProgram, tokenProgram, associatedTokenProgram, metadataProgram |
| **transferCrossChain** | collection, authority, nftMint, nftTokenAccount, gateway, gatewayPda, payer, tokenProgram, systemProgram |
| **onCall** | collection, collectionMint, gateway, gatewayPda, nftMint, nftTokenAccount, recipient, nftMetadata, payer, rent, systemProgram, tokenProgram, associatedTokenProgram, metadataProgram |

## Technical Specifications

### Program Metrics
- **Compute Usage**: ~23,819 / 200,000 units (optimized)
- **Program Size**: 348,920 bytes
- **Deployment Cost**: ~2.43 SOL
- **Transaction Fees**: ~0.000005 SOL per operation

### Dependencies
```json
{
  "@coral-xyz/anchor": "^0.29.0",
  "@solana/web3.js": "^1.95.4",
  "@solana/spl-token": "^0.4.9",
  "@metaplex-foundation/mpl-token-metadata": "^3.3.0"
}
```

## Project Structure

```
contracts/nft/contracts/solana/
├── programs/
│   └── universal_nft/
│       ├── src/
│       │   └── lib.rs              # Main program logic
│       └── target/
│           └── idl/
│               └── universal_nft.json  # Program IDL
├── scripts/
│   ├── local-solana.ts            # TypeScript test suite
│   ├── test-direct.js             # JavaScript test
│   ├── test-full-cross-chain.js  # Cross-chain test
│   └── test-on-call.js            # Gateway simulation
├── deployment.json                # Deployment configuration
├── Anchor.toml                    # Anchor configuration
└── package.json                   # Node dependencies
```

## Contributing

Contributions are welcome! Please ensure:
1. Tests pass with `npx ts-node scripts/local-solana.ts`
2. Anchor version 0.29.0 is used
3. All cross-chain functions are tested

## License

MIT

## Contact

For questions or issues, please open a GitHub issue or contact the ZetaChain team.

---

*Built with ❤️ for cross-chain interoperability*
