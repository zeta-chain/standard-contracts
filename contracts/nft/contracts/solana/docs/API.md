# Universal NFT Program API Documentation

## Table of Contents

1. [Overview](#overview)
2. [Program Information](#program-information)
3. [Account Structures](#account-structures)
4. [Instructions Reference](#instructions-reference)
5. [NFT Origin System](#nft-origin-system)
6. [Cross-Chain Message Formats](#cross-chain-message-formats)
7. [Integration Guides](#integration-guides)
8. [Error Codes](#error-codes)
9. [Events](#events)
10. [Security Considerations](#security-considerations)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)

## Overview

The Universal NFT program enables seamless cross-chain NFT transfers between Solana and other blockchains through ZetaChain's messaging protocol. The program implements a sophisticated **NFT Origin System** that maintains chain-agnostic NFT identity and metadata preservation across all supported networks.

### Key Features

- **🔗 Chain-Agnostic Identity**: Every NFT maintains its universal identity across all chains
- **📄 Metadata Preservation**: 100% fidelity of original metadata throughout cross-chain journey
- **🏠 Origin Tracking**: Complete chain-of-origin tracking for all NFTs
- **🔄 Two-Scenario Intelligence**: Automatic detection and handling of returning vs. first-time NFTs
- **⚡ Deterministic Security**: Cryptographically secure token ID generation
- **🎯 Complete Compatibility**: Full integration with existing EVM Universal NFT contracts

### Supported Networks

- **Solana** (Mainnet: 101, Testnet: 102, Devnet: 103)
- **Ethereum** (Mainnet: 1, Sepolia: 11155111)
- **Base** (Mainnet: 8453, Sepolia: 84532)
- **BNB Chain** (Mainnet: 56, Testnet: 97)
- **ZetaChain** (Mainnet: 7000, Testnet: 7001)

## Program Information

### Program ID
```
GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip
```

### Dependencies
- **Anchor Framework**: 0.29.0
- **Solana Program Library**: 1.18.26
- **Metaplex Token Metadata**: 4.1.2
- **ZetaChain Gateway**: ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis

### Resource Requirements
- **Program Size**: 348,920 bytes
- **Compute Usage**: ~28,000 units per operation (including origin tracking)
- **Storage per NFT**: ~388 bytes (collection + origin PDA)
- **Rent per NFT**: ~0.00388 SOL (collection + origin PDA)

## Account Structures

### Collection Account

The Collection account serves as the root configuration for a Universal NFT collection with enhanced origin tracking capabilities.

```rust
#[account]
#[derive(InitSpace)]
pub struct Collection {
    pub authority: Pubkey,              // Program authority (32 bytes)
    #[max_len(32)]
    pub name: String,                   // Collection name (max 32 chars)
    #[max_len(10)]
    pub symbol: String,                 // Collection symbol (max 10 chars)
    #[max_len(200)]
    pub uri: String,                    // Collection metadata URI (max 200 chars)
    pub tss_address: [u8; 20],         // TSS signature verification (20 bytes)
    pub gateway_address: Option<Pubkey>, // ZetaChain gateway address (33 bytes)
    pub universal_address: Option<Pubkey>, // Universal contract on ZetaChain (33 bytes)
    pub next_token_id: u64,            // Next token ID to mint (8 bytes)
    pub nonce: u64,                    // Replay protection (8 bytes)
    pub total_minted: u64,             // 🆕 Track total NFTs minted (8 bytes)
    pub solana_native_count: u64,      // 🆕 Track Solana-native NFTs (8 bytes)
    pub bump: u8,                      // PDA bump (1 byte)
}
```

**PDA Seeds**: `[b"collection", authority, name]`

**Size**: ~203 bytes + string lengths

#### Helper Methods
```rust
impl Collection {
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (self.total_minted, self.solana_native_count, self.total_minted - self.solana_native_count)
    }
    
    pub fn increment_total_minted(&mut self) -> Result<()> {
        self.total_minted = self.total_minted
            .checked_add(1)
            .ok_or(UniversalNftError::InvalidTokenId)?;
        Ok(())
    }
    
    pub fn increment_solana_native_count(&mut self) -> Result<()> {
        self.solana_native_count = self.solana_native_count
            .checked_add(1)
            .ok_or(UniversalNftError::InvalidTokenId)?;
        Ok(())
    }
}
```

### NFT Origin Account

The NFT Origin account is the cornerstone of the origin system, tracking the complete identity and history of each NFT across all chains.

```rust
#[account]
#[derive(InitSpace)]
pub struct NftOrigin {
    pub original_mint: Pubkey,         // 🆕 Stores the original mint key (32 bytes)
    pub token_id: u64,                 // 🆕 The universal token ID (8 bytes)
    pub collection: Pubkey,            // 🆕 Reference to the collection (32 bytes)
    pub chain_of_origin: u64,          // 🆕 Chain where NFT was first minted (8 bytes)
    pub created_at: i64,               // 🆕 Timestamp of creation (8 bytes)
    #[max_len(200)]
    pub metadata_uri: String,          // 🆕 Original metadata URI (max 200 chars)
    pub bump: u8,                      // 🆕 PDA bump (1 byte)
}
```

**PDA Seeds**: `[b"nft_origin", token_id.to_le_bytes()]`

**Size**: ~293 bytes + metadata URI length

#### Helper Methods
```rust
impl NftOrigin {
    pub fn seeds(token_id: u64) -> Vec<Vec<u8>> {
        vec![
            b"nft_origin".to_vec(),
            token_id.to_le_bytes().to_vec(),
        ]
    }
    
    pub fn validate_token_id(&self, expected_token_id: u64) -> Result<()> {
        require_eq!(
            self.token_id,
            expected_token_id,
            UniversalNftError::InvalidTokenId
        );
        Ok(())
    }
    
    pub fn is_solana_native(&self) -> bool {
        matches!(self.chain_of_origin, 101 | 102 | 103)
    }
    
    pub fn get_origin_chain_name(&self) -> &'static str {
        match self.chain_of_origin {
            101 => "Solana Mainnet",
            102 => "Solana Testnet", 
            103 => "Solana Devnet",
            11155111 => "Ethereum Sepolia",
            84532 => "Base Sepolia",
            97 => "BSC Testnet",
            7001 => "ZetaChain Testnet",
            _ => "Unknown Chain",
        }
    }
}
```

### Connected Account

The Connected account maps blockchain networks to their corresponding NFT contract addresses.

```rust
#[account]
#[derive(InitSpace)]
pub struct Connected {
    pub collection: Pubkey,            // Parent collection (32 bytes)
    #[max_len(32)]
    pub chain_id: Vec<u8>,            // Target chain ID (max 32 bytes)
    #[max_len(64)]
    pub contract_address: Vec<u8>,     // Contract address on target chain (max 64 bytes)
    pub bump: u8,                      // PDA bump (1 byte)
}
```

**PDA Seeds**: `[b"connected", collection, chain_id]`

**Size**: ~64 bytes + chain_id + contract_address lengths

## Instructions Reference

### initialize_collection

Initializes a new Universal NFT collection with origin tracking capabilities.

#### Function Signature
```rust
pub fn initialize_collection(
    ctx: Context<InitializeCollection>,
    name: String,
    symbol: String,
    uri: String,
    tss_address: [u8; 20]
) -> Result<()>
```

#### Parameters
- `name`: Collection name (max 32 characters)
- `symbol`: Collection symbol (max 10 characters)
- `uri`: Collection metadata URI (max 200 characters)
- `tss_address`: TSS signature verification address (20 bytes)

#### Required Accounts
```rust
#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + Collection::INIT_SPACE,
        seeds = [b"collection", authority.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub collection: Account<'info, Collection>,
    
    #[account(mut)]
    pub collection_mint: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        associated_token::mint = collection_mint,
        associated_token::authority = collection
    )]
    pub collection_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Metaplex program
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
```

#### Example Usage
```typescript
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';

const authority = Keypair.generate();
const collectionMint = Keypair.generate();
const collectionName = "Universal Test Collection";
const collectionSymbol = "UTC";
const collectionUri = "https://example.com/collection.json";
const tssAddress = new Uint8Array(20).fill(1); // Example TSS address

// Derive collection PDA
const [collectionPda, collectionBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("collection"),
    authority.publicKey.toBuffer(),
    Buffer.from(collectionName, 'utf8')
  ],
  program.programId
);

// Derive associated token account
const collectionTokenAccount = getAssociatedTokenAddressSync(
  collectionMint.publicKey,
  collectionPda,
  true
);

// Derive metadata account
const [collectionMetadata] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    collectionMint.publicKey.toBuffer(),
  ],
  TOKEN_METADATA_PROGRAM_ID
);

const tx = await program.methods
  .initializeCollection(
    collectionName,
    collectionSymbol,
    collectionUri,
    Array.from(tssAddress)
  )
  .accounts({
    authority: authority.publicKey,
    collection: collectionPda,
    collectionMint: collectionMint.publicKey,
    collectionTokenAccount: collectionTokenAccount,
    collectionMetadata: collectionMetadata,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    metadataProgram: TOKEN_METADATA_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([authority, collectionMint])
  .rpc();

console.log("Collection initialized:", tx);
console.log("Collection PDA:", collectionPda.toBase58());
```

#### Error Handling
```typescript
try {
  const tx = await program.methods.initializeCollection(/* ... */).rpc();
} catch (error) {
  if (error.code === 6001) {
    console.error("Invalid Authority");
  } else if (error.message.includes("already in use")) {
    console.error("Collection already exists");
  } else {
    console.error("Unknown error:", error);
  }
}
```

### mint_nft

Mints a new NFT within the collection with automatic origin PDA creation and deterministic token ID generation.

#### Function Signature
```rust
pub fn mint_nft(
    ctx: Context<MintNft>,
    name: String,
    symbol: String,
    uri: String
) -> Result<()>
```

#### Parameters
- `name`: NFT name (max 32 characters)
- `symbol`: NFT symbol (max 10 characters)
- `uri`: NFT metadata URI (max 200 characters)

#### Required Accounts
```rust
#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub collection: Account<'info, Collection>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub nft_mint: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Can be any account
    pub recipient: UncheckedAccount<'info>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = payer,
        space = 8 + NftOrigin::INIT_SPACE,
        seeds = [b"nft_origin", &collection.next_token_id.to_le_bytes()],
        bump
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Metaplex program
    pub metadata_program: UncheckedAccount<'info>,
}
```

#### Example Usage
```typescript
const nftMint = Keypair.generate();
const recipient = authority.publicKey; // or any other recipient
const nftName = "Universal NFT #1";
const nftSymbol = "UN1";
const nftUri = "https://example.com/nft/1.json";

// Get collection data to determine next token ID
const collectionData = await program.account.collection.fetch(collectionPda);
const nextTokenId = collectionData.nextTokenId;

// Derive origin PDA using next token ID
const [nftOriginPda, originBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("nft_origin"),
    nextTokenId.toArrayLike(Buffer, 'le', 8)
  ],
  program.programId
);

// Derive associated token account
const nftTokenAccount = getAssociatedTokenAddressSync(
  nftMint.publicKey,
  recipient,
  false
);

// Derive metadata account
const [nftMetadata] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    nftMint.publicKey.toBuffer(),
  ],
  TOKEN_METADATA_PROGRAM_ID
);

const tx = await program.methods
  .mintNft(nftName, nftSymbol, nftUri)
  .accounts({
    collection: collectionPda,
    authority: authority.publicKey,
    nftMint: nftMint.publicKey,
    nftTokenAccount: nftTokenAccount,
    recipient: recipient,
    nftMetadata: nftMetadata,
    nftOrigin: nftOriginPda,
    payer: authority.publicKey,
    rent: SYSVAR_RENT_PUBKEY,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    metadataProgram: TOKEN_METADATA_PROGRAM_ID,
  })
  .signers([authority, nftMint])
  .rpc();

console.log("NFT minted:", tx);
console.log("NFT mint:", nftMint.publicKey.toBase58());
console.log("Origin PDA:", nftOriginPda.toBase58());

// Verify origin PDA creation
const originData = await program.account.nftOrigin.fetch(nftOriginPda);
console.log("Token ID:", originData.tokenId.toString());
console.log("Origin chain:", originData.chainOfOrigin.toString());
console.log("Is Solana native:", originData.chainOfOrigin.toNumber() === 103);
```

#### Token ID Generation

The program uses deterministic token ID generation to ensure uniqueness:

```rust
// Deterministic token ID generation
let token_id = keccak256([
    mint_pubkey.to_bytes(),     // 32 bytes - unique mint address
    block_number.to_le_bytes(), // 8 bytes - current block number
    next_token_id.to_le_bytes() // 8 bytes - collection sequence
]).to_u64();
```

### transfer_cross_chain

Initiates a cross-chain NFT transfer with complete origin information preservation.

#### Function Signature
```rust
pub fn transfer_cross_chain(
    ctx: Context<TransferCrossChain>,
    destination_chain_id: u64,
    recipient: Vec<u8>
) -> Result<()>
```

#### Parameters
- `destination_chain_id`: Target blockchain chain ID
- `recipient`: Recipient address on target chain (20 bytes for EVM, 32 bytes for Solana)

#### Required Accounts
```rust
#[derive(Accounts)]
pub struct TransferCrossChain<'info> {
    pub collection: Account<'info, Collection>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = owner
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Validated by Metaplex
    pub nft_metadata: UncheckedAccount<'info>,
    
    pub nft_origin: Account<'info, NftOrigin>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Gateway PDA
    pub gateway_pda: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
```

#### Example Usage
```typescript
const destinationChainId = 11155111; // Ethereum Sepolia
const recipientAddress = "0x742d35Cc6634C0532925a3b8D4C9db96DfbB8E24";
const recipientBytes = Buffer.from(recipientAddress.slice(2), 'hex'); // Remove 0x prefix

// Get NFT origin data
const originData = await program.account.nftOrigin.fetch(nftOriginPda);
console.log("Transferring NFT with origin chain:", originData.chainOfOrigin.toString());
console.log("Original metadata URI:", originData.metadataUri);

// Derive gateway PDA
const [gatewayPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("meta")],
  ZETACHAIN_GATEWAY_PROGRAM_ID
);

const tx = await program.methods
  .transferCrossChain(
    new BN(destinationChainId),
    Array.from(recipientBytes)
  )
  .accounts({
    collection: collectionPda,
    owner: owner.publicKey,
    nftMint: nftMint.publicKey,
    nftTokenAccount: nftTokenAccount,
    nftMetadata: nftMetadata,
    nftOrigin: nftOriginPda,
    collectionMint: collectionMint.publicKey,
    gatewayPda: gatewayPda,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([owner])
  .rpc();

console.log("Cross-chain transfer initiated:", tx);
console.log("Destination chain:", destinationChainId);
console.log("Recipient:", recipientAddress);
```

#### Address Format Conversion
```typescript
// Convert addresses for different chains
function convertAddressForChain(address: string, chainId: number): Buffer {
  if (isEvmChain(chainId)) {
    // EVM chains use 20-byte addresses
    return Buffer.from(address.slice(2), 'hex');
  } else if (isSolanaChain(chainId)) {
    // Solana uses 32-byte addresses
    return new PublicKey(address).toBuffer();
  } else {
    throw new Error(`Unsupported chain: ${chainId}`);
  }
}

function isEvmChain(chainId: number): boolean {
  return [1, 56, 137, 8453, 42161, 10, 11155111, 97, 80001, 84532].includes(chainId);
}

function isSolanaChain(chainId: number): boolean {
  return [101, 102, 103].includes(chainId);
}
```

### on_call

Processes incoming NFT transfers from other chains with intelligent two-scenario handling based on the NFT Origin System.

#### Function Signature
```rust
pub fn on_call(
    ctx: Context<OnCall>,
    sender: [u8; 20],
    source_chain_id: u64,
    message: Vec<u8>,
    nonce: u64
) -> Result<()>
```

#### Parameters
- `sender`: Original sender address (TSS verified, 20 bytes)
- `source_chain_id`: Source blockchain chain ID
- `message`: Cross-chain message data (ABI or Borsh encoded)
- `nonce`: Replay protection nonce

#### Required Accounts
```rust
#[derive(Accounts)]
pub struct OnCall<'info> {
    #[account(mut)]
    pub collection: Account<'info, Collection>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: ZetaChain gateway program
    pub gateway: UncheckedAccount<'info>,
    
    /// CHECK: Gateway PDA
    pub gateway_pda: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub nft_mint: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Can be any account
    pub recipient: UncheckedAccount<'info>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + NftOrigin::INIT_SPACE,
        seeds = [b"nft_origin", &decode_token_id_from_message(&message)?],
        bump
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Metaplex program
    pub metadata_program: UncheckedAccount<'info>,
}
```

#### Two-Scenario Logic

The `on_call` function implements intelligent handling based on whether the NFT has been on Solana before:

**Scenario A: NFT Returning to Solana**
```rust
if origin_pda_exists {
    // Fetch original mint key and metadata from existing origin PDA
    let origin_account = fetch_origin_pda(token_id)?;
    
    // Create new mint but link to original metadata
    create_mint_with_preserved_identity(
        new_mint,
        recipient,
        origin_account.metadata_uri, // Use original metadata
        origin_account.token_id,
        origin_account.original_mint
    )?;
    
    // Emit returning event
    emit!(NftReturningToSolana { ... });
}
```

**Scenario B: NFT First Time on Solana**
```rust
if !origin_pda_exists {
    // Create new origin PDA with token_id from source chain
    let origin_pda = create_origin_pda(
        token_id,
        source_chain_id,
        metadata_uri
    )?;
    
    // Store origin chain information
    origin_pda.chain_of_origin = source_chain_id;
    
    // Create new mint and metadata from cross-chain message data
    create_mint_from_cross_chain_data(new_mint, recipient, metadata_uri)?;
    
    // Emit origin creation event
    emit!(NftOriginCreated { ... });
}
```

#### Example Usage (Gateway Simulation)
```typescript
// This function is typically called by the ZetaChain gateway
// For testing, we can simulate the call

const senderAddress = new Uint8Array(20).fill(1); // Example sender
const sourceChainId = 11155111; // Ethereum Sepolia
const nonce = 1;

// Create cross-chain message
const tokenId = new BN(12345);
const metadataUri = "https://example.com/nft/12345.json";
const recipientPubkey = new PublicKey("11111111111111111111111111111112");

const message = createCrossChainMessage({
  tokenId: tokenId.toNumber(),
  uri: metadataUri,
  recipient: recipientPubkey.toBuffer(),
  sender: senderAddress,
  originChain: sourceChainId,
});

// Derive origin PDA
const [nftOriginPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("nft_origin"),
    tokenId.toArrayLike(Buffer, 'le', 8)
  ],
  program.programId
);

// Check if this is scenario A or B
let scenarioType: 'A' | 'B';
try {
  await program.account.nftOrigin.fetch(nftOriginPda);
  scenarioType = 'A'; // NFT returning to Solana
  console.log("Scenario A: NFT returning to Solana");
} catch {
  scenarioType = 'B'; // NFT first time on Solana
  console.log("Scenario B: NFT first time on Solana");
}

const newNftMint = Keypair.generate();
const nftTokenAccount = getAssociatedTokenAddressSync(
  newNftMint.publicKey,
  recipientPubkey,
  false
);

const [nftMetadata] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    newNftMint.publicKey.toBuffer(),
  ],
  TOKEN_METADATA_PROGRAM_ID
);

const tx = await program.methods
  .onCall(
    Array.from(senderAddress),
    new BN(sourceChainId),
    Array.from(message),
    new BN(nonce)
  )
  .accounts({
    collection: collectionPda,
    collectionMint: collectionMint.publicKey,
    gateway: ZETACHAIN_GATEWAY_PROGRAM_ID,
    gatewayPda: gatewayPda,
    nftMint: newNftMint.publicKey,
    nftTokenAccount: nftTokenAccount,
    recipient: recipientPubkey,
    nftMetadata: nftMetadata,
    nftOrigin: nftOriginPda,
    payer: payer.publicKey,
    rent: SYSVAR_RENT_PUBKEY,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    metadataProgram: TOKEN_METADATA_PROGRAM_ID,
  })
  .signers([payer, newNftMint])
  .rpc();

console.log("NFT received via on_call:", tx);
console.log("Scenario:", scenarioType);
console.log("New mint:", newNftMint.publicKey.toBase58());
```

### set_universal

Sets the universal contract address on ZetaChain for cross-chain operations.

#### Function Signature
```rust
pub fn set_universal(
    ctx: Context<SetUniversal>,
    universal_address: Pubkey
) -> Result<()>
```

#### Parameters
- `universal_address`: ZetaChain universal contract address

#### Required Accounts
```rust
#[derive(Accounts)]
pub struct SetUniversal<'info> {
    #[account(mut)]
    pub collection: Account<'info, Collection>,
    
    pub authority: Signer<'info>,
}
```

#### Example Usage
```typescript
const universalAddress = new PublicKey("UniversalContract1111111111111111111111111");

const tx = await program.methods
  .setUniversal(universalAddress)
  .accounts({
    collection: collectionPda,
    authority: authority.publicKey,
  })
  .signers([authority])
  .rpc();

console.log("Universal address set:", tx);
```

### set_connected

Links blockchain networks to their corresponding NFT contract addresses.

#### Function Signature
```rust
pub fn set_connected(
    ctx: Context<SetConnected>,
    chain_id: Vec<u8>,
    contract_address: Vec<u8>
) -> Result<()>
```

#### Parameters
- `chain_id`: Target blockchain chain ID (8 bytes)
- `contract_address`: Contract address on target chain

#### Required Accounts
```rust
#[derive(Accounts)]
pub struct SetConnected<'info> {
    pub collection: Account<'info, Collection>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Connected::INIT_SPACE,
        seeds = [b"connected", collection.key().as_ref(), &chain_id],
        bump
    )]
    pub connected: Account<'info, Connected>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
```

#### Example Usage
```typescript
// Connect to Ethereum Sepolia
const ethereumChainId = new BN(11155111);
const ethereumContract = "0x742d35Cc6634C0532925a3b8D4C9db96DfbB8E24";

const chainIdBytes = ethereumChainId.toArrayLike(Buffer, 'le', 8);
const contractBytes = Buffer.from(ethereumContract.slice(2), 'hex');

// Derive connected PDA
const [connectedPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("connected"),
    collectionPda.toBuffer(),
    chainIdBytes
  ],
  program.programId
);

const tx = await program.methods
  .setConnected(
    Array.from(chainIdBytes),
    Array.from(contractBytes)
  )
  .accounts({
    collection: collectionPda,
    connected: connectedPda,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([authority])
  .rpc();

console.log("Connected to Ethereum:", tx);

// Connect to Base Sepolia
const baseChainId = new BN(84532);
const baseContract = "0x742d35Cc6634C0532925a3b8D4C9db96DfbB8E24";

const baseChainIdBytes = baseChainId.toArrayLike(Buffer, 'le', 8);
const baseContractBytes = Buffer.from(baseContract.slice(2), 'hex');

const [baseConnectedPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("connected"),
    collectionPda.toBuffer(),
    baseChainIdBytes
  ],
  program.programId
);

const baseTx = await program.methods
  .setConnected(
    Array.from(baseChainIdBytes),
    Array.from(baseContractBytes)
  )
  .accounts({
    collection: collectionPda,
    connected: baseConnectedPda,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([authority])
  .rpc();

console.log("Connected to Base:", baseTx);
```

### on_revert

Handles failed cross-chain transfers by minting the NFT back to the original sender with origin information preservation.

#### Function Signature
```rust
pub fn on_revert(
    ctx: Context<OnRevert>,
    token_id: u64,
    uri: String,
    original_sender: Pubkey,
    refund_amount: u64
) -> Result<()>
```

#### Parameters
- `token_id`: Original token ID from origin system
- `uri`: NFT metadata URI (preserved from origin)
- `original_sender`: Original sender to receive reverted NFT
- `refund_amount`: Optional refund amount

#### Required Accounts
```rust
#[derive(Accounts)]
pub struct OnRevert<'info> {
    pub collection: Account<'info, Collection>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: ZetaChain gateway program
    pub gateway: UncheckedAccount<'info>,
    
    /// CHECK: Gateway PDA
    pub gateway_pda: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub nft_mint: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = original_sender
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Original sender
    pub original_sender: UncheckedAccount<'info>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + NftOrigin::INIT_SPACE,
        seeds = [b"nft_origin", &token_id.to_le_bytes()],
        bump
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Metaplex program
    pub metadata_program: UncheckedAccount<'info>,
}
```

#### Example Usage
```typescript
// This function is typically called by the ZetaChain gateway for failed transfers
const tokenId = new BN(12345);
const metadataUri = "https://example.com/nft/12345.json";
const originalSender = new PublicKey("11111111111111111111111111111112");
const refundAmount = new BN(0); // No refund in this example

const revertedNftMint = Keypair.generate();

// Derive origin PDA
const [nftOriginPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("nft_origin"),
    tokenId.toArrayLike(Buffer, 'le', 8)
  ],
  program.programId
);

const nftTokenAccount = getAssociatedTokenAddressSync(
  revertedNftMint.publicKey,
  originalSender,
  false
);

const [nftMetadata] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    revertedNftMint.publicKey.toBuffer(),
  ],
  TOKEN_METADATA_PROGRAM_ID
);

const tx = await program.methods
  .onRevert(
    tokenId,
    metadataUri,
    originalSender,
    refundAmount
  )
  .accounts({
    collection: collectionPda,
    collectionMint: collectionMint.publicKey,
    gateway: ZETACHAIN_GATEWAY_PROGRAM_ID,
    gatewayPda: gatewayPda,
    nftMint: revertedNftMint.publicKey,
    nftTokenAccount: nftTokenAccount,
    originalSender: originalSender,
    nftMetadata: nftMetadata,
    nftOrigin: nftOriginPda,
    payer: payer.publicKey,
    rent: SYSVAR_RENT_PUBKEY,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    metadataProgram: TOKEN_METADATA_PROGRAM_ID,
  })
  .signers([payer, revertedNftMint])
  .rpc();

console.log("NFT reverted:", tx);
console.log("Reverted to:", originalSender.toBase58());
```

## NFT Origin System

The NFT Origin System is the cornerstone innovation of this Universal NFT implementation, solving the critical challenge of maintaining NFT identity across chains.

### Core Concepts

#### Universal Token ID
Every NFT receives a deterministic, globally unique token ID that remains constant across all chains:

```rust
// Deterministic token ID generation
let token_id = keccak256([
    mint_pubkey.to_bytes(),     // 32 bytes - unique mint address
    block_number.to_le_bytes(), // 8 bytes - current block number
    next_token_id.to_le_bytes() // 8 bytes - collection sequence
]).to_u64();
```

#### Origin PDA Structure
Each NFT's origin information is stored in a dedicated PDA:

```typescript
// Origin PDA derivation
const [originPda, bump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("nft_origin"),
    tokenId.toArrayLike(Buffer, 'le', 8)
  ],
  program.programId
);
```

#### Two-Scenario Reception Logic

**Scenario A: NFT Returning to Solana**
- Origin PDA exists for the token ID
- Original metadata and mint reference are preserved
- New mint is created but linked to original identity
- `NftReturningToSolana` event is emitted

**Scenario B: NFT First Time on Solana**
- Origin PDA doesn't exist for the token ID
- New origin PDA is created with source chain information
- NFT is marked as non-Solana-native
- `NftOriginCreated` event is emitted

### Origin System Utilities

#### Query Origin Information
```typescript
async function getNftOriginInfo(
  program: Program,
  tokenId: number
): Promise<NftOriginInfo | null> {
  try {
    const [originPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        new BN(tokenId).toArrayLike(Buffer, 'le', 8)
      ],
      program.programId
    );

    const originAccount = await program.account.nftOrigin.fetch(originPda);
    
    return {
      tokenId: originAccount.tokenId.toNumber(),
      originalMint: originAccount.originalMint,
      chainOfOrigin: originAccount.chainOfOrigin.toNumber(),
      chainName: getChainName(originAccount.chainOfOrigin.toNumber()),
      metadataUri: originAccount.metadataUri,
      createdAt: new Date(originAccount.createdAt.toNumber() * 1000),
      isSolanaNative: isSolanaNative(originAccount.chainOfOrigin.toNumber()),
    };
  } catch (error) {
    console.error(`Failed to fetch origin info for token ${tokenId}:`, error);
    return null;
  }
}

interface NftOriginInfo {
  tokenId: number;
  originalMint: PublicKey;
  chainOfOrigin: number;
  chainName: string;
  metadataUri: string;
  createdAt: Date;
  isSolanaNative: boolean;
}
```

#### Track NFT Journey
```typescript
async function trackNftJourney(
  program: Program,
  tokenId: number
): Promise<NftJourney> {
  const events = await getEventsForTokenId(program, tokenId);
  
  const journey: NftJourney = {
    tokenId,
    originChain: 0,
    currentChain: 103, // Solana devnet
    chainsVisited: [],
    transferCount: 0,
    cycleCount: 0,
    events: [],
  };

  for (const event of events) {
    switch (event.name) {
      case 'NftOriginCreated':
        journey.originChain = event.data.originChain;
        journey.chainsVisited.push(event.data.originChain);
        break;
      case 'TokenTransferWithOrigin':
        journey.transferCount++;
        journey.chainsVisited.push(event.data.destinationChainId);
        if (event.data.isReturningToOrigin) {
          journey.cycleCount++;
        }
        break;
      case 'TokenTransferReceivedWithOrigin':
        journey.currentChain = 103; // Back on Solana
        break;
    }
    journey.events.push(event);
  }

  return journey;
}

interface NftJourney {
  tokenId: number;
  originChain: number;
  currentChain: number;
  chainsVisited: number[];
  transferCount: number;
  cycleCount: number;
  events: any[];
}
```

#### Validate Origin Integrity
```typescript
async function validateOriginIntegrity(
  program: Program,
  originPda: PublicKey,
  expectedTokenId: number
): Promise<boolean> {
  try {
    const originAccount = await program.account.nftOrigin.fetch(originPda);
    
    // Validate token ID
    if (originAccount.tokenId.toNumber() !== expectedTokenId) {
      console.error(`Token ID mismatch: expected ${expectedTokenId}, got ${originAccount.tokenId}`);
      return false;
    }
    
    // Validate chain of origin
    if (!isValidChainId(originAccount.chainOfOrigin.toNumber())) {
      console.error(`Invalid origin chain: ${originAccount.chainOfOrigin}`);
      return false;
    }
    
    // Validate metadata URI
    if (!originAccount.metadataUri || originAccount.metadataUri.length === 0) {
      console.error("Empty metadata URI");
      return false;
    }
    
    // Validate timestamp
    if (originAccount.createdAt.toNumber() <= 0) {
      console.error("Invalid creation timestamp");
      return false;
    }
    
    return true;
  } catch (error) {
    console.error(`Failed to validate origin data: ${error.message}`);
    return false;
  }
}

function isValidChainId(chainId: number): boolean {
  const validChains = [101, 102, 103, 1, 56, 137, 8453, 42161, 10, 11155111, 97, 80001, 84532, 7000, 7001];
  return validChains.includes(chainId);
}
```

## Cross-Chain Message Formats

The Universal NFT program supports multiple message formats to ensure compatibility across different blockchain ecosystems.

### Enhanced Cross-Chain Message Structure

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EnhancedCrossChainMessage {
    pub token_id: u64,              // Universal token ID
    pub metadata_uri: String,       // NFT metadata URI
    pub recipient: Vec<u8>,         // Recipient address
    pub sender: Vec<u8>,            // Original sender address
    pub origin_chain: u64,          // 🆕 Chain where NFT was first minted
    pub original_mint: Pubkey,      // 🆕 Original mint key (Solana) or contract (EVM)
    pub is_solana_native: bool,     // 🆕 Whether NFT originated on Solana
    pub created_at: i64,            // 🆕 Original creation timestamp
    pub cycle_count: u64,           // 🆕 Number of cross-chain cycles
    pub is_returning_to_origin: bool, // 🆕 Whether returning to origin chain
}
```

### Message Encoding/Decoding

#### ABI Encoding (for EVM chains)
```typescript
function encodeMessageForEvm(message: EnhancedCrossChainMessage): Buffer {
  // ABI encode: (uint256,string,address,address,uint256,bytes32,bool,int64,uint256,bool)
  const encoded = ethers.utils.defaultAbiCoder.encode(
    [
      'uint256',  // token_id
      'string',   // metadata_uri
      'address',  // recipient
      'address',  // sender
      'uint256',  // origin_chain
      'bytes32',  // original_mint
      'bool',     // is_solana_native
      'int64',    // created_at
      'uint256',  // cycle_count
      'bool'      // is_returning_to_origin
    ],
    [
      message.tokenId,
      message.metadataUri,
      message.recipient,
      message.sender,
      message.originChain,
      message.originalMint,
      message.isSolanaNative,
      message.createdAt,
      message.cycleCount,
      message.isReturningToOrigin
    ]
  );
  
  return Buffer.from(encoded.slice(2), 'hex');
}

function decodeAbiMessage(data: Buffer): EnhancedCrossChainMessage {
  const decoded = ethers.utils.defaultAbiCoder.decode(
    [
      'uint256', 'string', 'address', 'address', 'uint256',
      'bytes32', 'bool', 'int64', 'uint256', 'bool'
    ],
    '0x' + data.toString('hex')
  );
  
  return {
    tokenId: decoded[0].toNumber(),
    metadataUri: decoded[1],
    recipient: Buffer.from(decoded[2].slice(2), 'hex'),
    sender: Buffer.from(decoded[3].slice(2), 'hex'),
    originChain: decoded[4].toNumber(),
    originalMint: new PublicKey(decoded[5]),
    isSolanaNative: decoded[6],
    createdAt: decoded[7].toNumber(),
    cycleCount: decoded[8].toNumber(),
    isReturningToOrigin: decoded[9]
  };
}
```

#### Borsh Encoding (for Solana)
```typescript
import { serialize, deserialize } from 'borsh';

class EnhancedCrossChainMessage {
  tokenId: number;
  metadataUri: string;
  recipient: Uint8Array;
  sender: Uint8Array;
  originChain: number;
  originalMint: Uint8Array;
  isSolanaNative: boolean;
  createdAt: number;
  cycleCount: number;
  isReturningToOrigin: boolean;

  constructor(fields: any) {
    Object.assign(this, fields);
  }

  static schema = new Map([
    [EnhancedCrossChainMessage, {
      kind: 'struct',
      fields: [
        ['tokenId', 'u64'],
        ['metadataUri', 'string'],
        ['recipient', ['u8']],
        ['sender', ['u8']],
        ['originChain', 'u64'],
        ['originalMint', [32]],
        ['isSolanaNative', 'u8'],
        ['createdAt', 'i64'],
        ['cycleCount', 'u64'],
        ['isReturningToOrigin', 'u8']
      ]
    }]
  ]);

  serialize(): Buffer {
    return Buffer.from(serialize(EnhancedCrossChainMessage.schema, this));
  }

  static deserialize(data: Buffer): EnhancedCrossChainMessage {
    return deserialize(EnhancedCrossChainMessage.schema, EnhancedCrossChainMessage, data);
  }
}
```

#### Message Format Detection
```typescript
function decodeCrossChainMessage(messageData: Buffer): EnhancedCrossChainMessage {
  try {
    // Try ABI decoding first (from EVM)
    return decodeAbiMessage(messageData);
  } catch (abiError) {
    try {
      // Fall back to Borsh decoding (from Solana)
      return EnhancedCrossChainMessage.deserialize(messageData);
    } catch (borshError) {
      throw new Error(`Failed to decode message: ABI(${abiError.message}), Borsh(${borshError.message})`);
    }
  }
}
```

### Address Format Conversion

```typescript
// Convert between different address formats
function convertAddressFormat(address: string | Buffer, targetChain: number): Buffer {
  if (isEvmChain(targetChain)) {
    // EVM chains require 20-byte addresses
    if (typeof address === 'string') {
      if (address.startsWith('0x')) {
        return Buffer.from(address.slice(2), 'hex');
      } else {
        // Solana address to EVM
        const pubkey = new PublicKey(address);
        return pubkey.toBuffer().slice(12, 32); // Take last 20 bytes
      }
    } else {
      return address.length === 20 ? address : address.slice(12, 32);
    }
  } else if (isSolanaChain(targetChain)) {
    // Solana requires 32-byte addresses
    if (typeof address === 'string') {
      if (address.startsWith('0x')) {
        // EVM address to Solana
        const evmBytes = Buffer.from(address.slice(2), 'hex');
        const padded = Buffer.alloc(32);
        evmBytes.copy(padded, 12);
        return padded;
      } else {
        return new PublicKey(address).toBuffer();
      }
    } else {
      return address.length === 32 ? address : (() => {
        const padded = Buffer.alloc(32);
        address.copy(padded, 12);
        return padded;
      })();
    }
  } else {
    throw new Error(`Unsupported target chain: ${targetChain}`);
  }
}
```

## Integration Guides

### Frontend Integration

#### React Component Example
```typescript
import React, { useState, useEffect } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';

interface UniversalNftComponentProps {
  programId: PublicKey;
  collectionPda: PublicKey;
}

export const UniversalNftComponent: React.FC<UniversalNftComponentProps> = ({
  programId,
  collectionPda
}) => {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [program, setProgram] = useState<Program | null>(null);
  const [nfts, setNfts] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (connection && wallet.wallet) {
      const provider = new AnchorProvider(connection, wallet as any, {});
      const program = new Program(idl, programId, provider);
      setProgram(program);
    }
  }, [connection, wallet, programId]);

  const mintNft = async (name: string, symbol: string, uri: string) => {
    if (!program || !wallet.publicKey) return;

    setLoading(true);
    try {
      const nftMint = Keypair.generate();
      
      // Get next token ID
      const collectionData = await program.account.collection.fetch(collectionPda);
      const nextTokenId = collectionData.nextTokenId;

      // Derive origin PDA
      const [originPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("nft_origin"),
          nextTokenId.toArrayLike(Buffer, 'le', 8)
        ],
        program.programId
      );

      const tx = await program.methods
        .mintNft(name, symbol, uri)
        .accounts({
          collection: collectionPda,
          authority: wallet.publicKey,
          nftMint: nftMint.publicKey,
          recipient: wallet.publicKey,
          nftOrigin: originPda,
          // ... other accounts
        })
        .signers([nftMint])
        .rpc();

      console.log("NFT minted:", tx);
      
      // Refresh NFT list
      await loadNfts();
    } catch (error) {
      console.error("Failed to mint NFT:", error);
    } finally {
      setLoading(false);
    }
  };

  const transferToChain = async (
    nftMint: PublicKey,
    originPda: PublicKey,
    chainId: number,
    recipient: string
  ) => {
    if (!program || !wallet.publicKey) return;

    setLoading(true);
    try {
      const recipientBytes = convertAddressFormat(recipient, chainId);

      const tx = await program.methods
        .transferCrossChain(new BN(chainId), Array.from(recipientBytes))
        .accounts({
          collection: collectionPda,
          owner: wallet.publicKey,
          nftMint: nftMint,
          nftOrigin: originPda,
          // ... other accounts
        })
        .rpc();

      console.log("Cross-chain transfer initiated:", tx);
      
      // Refresh NFT list
      await loadNfts();
    } catch (error) {
      console.error("Failed to transfer NFT:", error);
    } finally {
      setLoading(false);
    }
  };

  const loadNfts = async () => {
    if (!program || !wallet.publicKey) return;

    try {
      // Get all token accounts owned by user
      const tokenAccounts = await connection.getTokenAccountsByOwner(
        wallet.publicKey,
        { programId: TOKEN_PROGRAM_ID }
      );

      const nftData = [];
      for (const { account } of tokenAccounts) {
        const tokenAccountData = AccountLayout.decode(account.data);
        if (tokenAccountData.amount.toString() === '1') {
          // This is likely an NFT
          try {
            // Try to get origin data
            const [originPda] = PublicKey.findProgramAddressSync(
              [Buffer.from("nft_origin"), /* token_id */],
              program.programId
            );
            
            const originData = await program.account.nftOrigin.fetch(originPda);
            nftData.push({
              mint: new PublicKey(tokenAccountData.mint),
              originPda,
              originData,
            });
          } catch {
            // Not a Universal NFT
          }
        }
      }

      setNfts(nftData);
    } catch (error) {
      console.error("Failed to load NFTs:", error);
    }
  };

  useEffect(() => {
    loadNfts();
  }, [program, wallet.publicKey]);

  return (
    <div className="universal-nft-component">
      <h2>Universal NFT Collection</h2>
      
      <div className="mint-section">
        <h3>Mint New NFT</h3>
        <MintForm onMint={mintNft} loading={loading} />
      </div>

      <div className="nft-list">
        <h3>Your NFTs</h3>
        {nfts.map((nft, index) => (
          <NftCard
            key={index}
            nft={nft}
            onTransfer={transferToChain}
            loading={loading}
          />
        ))}
      </div>
    </div>
  );
};

const MintForm: React.FC<{
  onMint: (name: string, symbol: string, uri: string) => void;
  loading: boolean;
}> = ({ onMint, loading }) => {
  const [name, setName] = useState('');
  const [symbol, setSymbol] = useState('');
  const [uri, setUri] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onMint(name, symbol, uri);
    setName('');
    setSymbol('');
    setUri('');
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="NFT Name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        required
      />
      <input
        type="text"
        placeholder="Symbol"
        value={symbol}
        onChange={(e) => setSymbol(e.target.value)}
        required
      />
      <input
        type="url"
        placeholder="Metadata URI"
        value={uri}
        onChange={(e) => setUri(e.target.value)}
        required
      />
      <button type="submit" disabled={loading}>
        {loading ? 'Minting...' : 'Mint NFT'}
      </button>
    </form>
  );
};

const NftCard: React.FC<{
  nft: any;
  onTransfer: (mint: PublicKey, originPda: PublicKey, chainId: number, recipient: string) => void;
  loading: boolean;
}> = ({ nft, onTransfer, loading }) => {
  const [transferChain, setTransferChain] = useState('11155111'); // Ethereum Sepolia
  const [recipient, setRecipient] = useState('');

  const handleTransfer = () => {
    onTransfer(nft.mint, nft.originPda, parseInt(transferChain), recipient);
    setRecipient('');
  };

  return (
    <div className="nft-card">
      <h4>Token ID: {nft.originData.tokenId.toString()}</h4>
      <p>Origin Chain: {nft.originData.chainOfOrigin.toString()}</p>
      <p>Metadata: {nft.originData.metadataUri}</p>
      <p>Is Solana Native: {nft.originData.chainOfOrigin.toNumber() === 103 ? 'Yes' : 'No'}</p>
      
      <div className="transfer-section">
        <select
          value={transferChain}
          onChange={(e) => setTransferChain(e.target.value)}
        >
          <option value="11155111">Ethereum Sepolia</option>
          <option value="84532">Base Sepolia</option>
          <option value="97">BSC Testnet</option>
        </select>
        <input
          type="text"
          placeholder="Recipient Address"
          value={recipient}
          onChange={(e) => setRecipient(e.target.value)}
        />
        <button onClick={handleTransfer} disabled={loading || !recipient}>
          Transfer
        </button>
      </div>
    </div>
  );
};
```

### Backend Service Integration

#### Node.js Service
```typescript
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import express from 'express';

class UniversalNftService {
  private connection: Connection;
  private program: Program;
  private authority: Keypair;
  private collectionPda: PublicKey;

  constructor(
    rpcUrl: string,
    programId: PublicKey,
    authorityKeypair: Keypair,
    collectionPda: PublicKey
  ) {
    this.connection = new Connection(rpcUrl);
    this.authority = authorityKeypair;
    this.collectionPda = collectionPda;

    const wallet = new Wallet(this.authority);
    const provider = new AnchorProvider(this.connection, wallet, {});
    this.program = new Program(idl, programId, provider);
  }

  async mintNftForUser(
    recipient: PublicKey,
    name: string,
    symbol: string,
    uri: string
  ): Promise<{ transaction: string; mint: PublicKey; tokenId: number; originPda: PublicKey }> {
    const nftMint = Keypair.generate();
    
    // Get next token ID
    const collectionData = await this.program.account.collection.fetch(this.collectionPda);
    const nextTokenId = collectionData.nextTokenId.toNumber();

    // Derive origin PDA
    const [originPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        new BN(nextTokenId).toArrayLike(Buffer, 'le', 8)
      ],
      this.program.programId
    );

    const tx = await this.program.methods
      .mintNft(name, symbol, uri)
      .accounts({
        collection: this.collectionPda,
        authority: this.authority.publicKey,
        nftMint: nftMint.publicKey,
        recipient: recipient,
        nftOrigin: originPda,
        // ... other accounts
      })
      .signers([nftMint])
      .rpc();

    return {
      transaction: tx,
      mint: nftMint.publicKey,
      tokenId: nextTokenId,
      originPda
    };
  }

  async handleIncomingTransfer(
    sender: string,
    sourceChainId: number,
    message: Buffer,
    nonce: number,
    recipient: PublicKey
  ): Promise<{ transaction: string; scenario: 'A' | 'B'; originPda: PublicKey }> {
    // Decode message
    const decodedMessage = decodeCrossChainMessage(message);
    
    // Derive origin PDA
    const [originPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        new BN(decodedMessage.tokenId).toArrayLike(Buffer, 'le', 8)
      ],
      this.program.programId
    );

    // Check scenario
    const originExists = await this.checkOriginPdaExists(originPda);
    const scenario = originExists ? 'A' : 'B';

    const newNftMint = Keypair.generate();
    
    const tx = await this.program.methods
      .onCall(
        Array.from(Buffer.from(sender.slice(2), 'hex')),
        new BN(sourceChainId),
        Array.from(message),
        new BN(nonce)
      )
      .accounts({
        collection: this.collectionPda,
        gateway: ZETACHAIN_GATEWAY_PROGRAM_ID,
        nftMint: newNftMint.publicKey,
        recipient: recipient,
        nftOrigin: originPda,
        // ... other accounts
      })
      .signers([newNftMint])
      .rpc();

    return { transaction: tx, scenario, originPda };
  }

  async getOriginInfo(tokenId: number): Promise<any> {
    const [originPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        new BN(tokenId).toArrayLike(Buffer, 'le', 8)
      ],
      this.program.programId
    );

    try {
      const originData = await this.program.account.nftOrigin.fetch(originPda);
      return {
        tokenId: originData.tokenId.toNumber(),
        originalMint: originData.originalMint.toBase58(),
        chainOfOrigin: originData.chainOfOrigin.toNumber(),
        metadataUri: originData.metadataUri,
        createdAt: new Date(originData.createdAt.toNumber() * 1000),
        isSolanaNative: originData.chainOfOrigin.toNumber() === 103,
      };
    } catch (error) {
      return null;
    }
  }

  async getCollectionStats(): Promise<any> {
    const collectionData = await this.program.account.collection.fetch(this.collectionPda);
    return {
      totalMinted: collectionData.totalMinted.toNumber(),
      solanaNativeCount: collectionData.solanaNativeCount.toNumber(),
      crossChainCount: collectionData.totalMinted.toNumber() - collectionData.solanaNativeCount.toNumber(),
      nextTokenId: collectionData.nextTokenId.toNumber(),
      nonce: collectionData.nonce.toNumber(),
    };
  }

  private async checkOriginPdaExists(originPda: PublicKey): Promise<boolean> {
    try {
      await this.program.account.nftOrigin.fetch(originPda);
      return true;
    } catch {
      return false;
    }
  }
}

// Express API
const app = express();
app.use(express.json());

const service = new UniversalNftService(
  process.env.RPC_URL!,
  new PublicKey(process.env.PROGRAM_ID!),
  Keypair.fromSecretKey(/* authority keypair */),
  new PublicKey(process.env.COLLECTION_PDA!)
);

app.post('/mint', async (req, res) => {
  try {
    const { recipient, name, symbol, uri } = req.body;
    const result = await service.mintNftForUser(
      new PublicKey(recipient),
      name,
      symbol,
      uri
    );
    res.json(result);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get('/origin/:tokenId', async (req, res) => {
  try {
    const tokenId = parseInt(req.params.tokenId);
    const originInfo = await service.getOriginInfo(tokenId);
    if (originInfo) {
      res.json(originInfo);
    } else {
      res.status(404).json({ error: 'Origin not found' });
    }
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get('/stats', async (req, res) => {
  try {
    const stats = await service.getCollectionStats();
    res.json(stats);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.listen(3000, () => {
  console.log('Universal NFT service running on port 3000');
});
```

### Wallet Integration

#### Custom Hook for Wallet Integration
```typescript
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { useCallback, useMemo } from 'react';
import { PublicKey, Transaction, sendAndConfirmTransaction } from '@solana/web3.js';
import { Program, AnchorProvider } from '@coral-xyz/anchor';

export function useUniversalNft(programId: PublicKey, collectionPda: PublicKey) {
  const { connection } = useConnection();
  const wallet = useWallet();

  const program = useMemo(() => {
    if (!wallet.wallet || !connection) return null;
    
    const provider = new AnchorProvider(connection, wallet as any, {
      commitment: 'confirmed',
    });
    return new Program(idl, programId, provider);
  }, [connection, wallet, programId]);

  const mintNft = useCallback(async (
    name: string,
    symbol: string,
    uri: string
  ) => {
    if (!program || !wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    // Implementation similar to previous examples
    // ...
  }, [program, wallet]);

  const transferCrossChain = useCallback(async (
    nftMint: PublicKey,
    originPda: PublicKey,
    destinationChainId: number,
    recipient: string
  ) => {
    if (!program || !wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    // Implementation similar to previous examples
    // ...
  }, [program, wallet]);

  const getOwnedNfts = useCallback(async () => {
    if (!program || !wallet.publicKey) return [];

    // Get all token accounts owned by user
    const tokenAccounts = await connection.getTokenAccountsByOwner(
      wallet.publicKey,
      { programId: TOKEN_PROGRAM_ID }
    );

    const nfts = [];
    for (const { account } of tokenAccounts) {
      // Check if this is a Universal NFT
      // Implementation to identify and fetch origin data
      // ...
    }

    return nfts;
  }, [program, wallet.publicKey, connection]);

  return {
    program,
    mintNft,
    transferCrossChain,
    getOwnedNfts,
    connected: !!wallet.connected,
    publicKey: wallet.publicKey,
  };
}
```

## Error Codes

The Universal NFT program defines comprehensive error codes for different failure scenarios:

```rust
#[error_code]
pub enum UniversalNftError {
    // General errors (6000-6099)
    #[msg("Invalid token ID")]
    InvalidTokenId = 6000,
    
    #[msg("Invalid authority")]
    InvalidAuthority = 6001,
    
    #[msg("Invalid chain ID")]
    InvalidChainId = 6002,
    
    #[msg("Invalid message format")]
    InvalidMessage = 6003,
    
    #[msg("Invalid nonce")]
    InvalidNonce = 6004,
    
    #[msg("Replay attack detected")]
    ReplayAttack = 6005,
    
    #[msg("Insufficient balance")]
    InsufficientBalance = 6006,
    
    #[msg("Invalid recipient address")]
    InvalidRecipient = 6007,
    
    #[msg("Invalid input")]
    InvalidInput = 6008,
    
    // Gateway errors (6100-6199)
    #[msg("Unauthorized gateway access")]
    UnauthorizedGateway = 6100,
    
    #[msg("Invalid TSS signature")]
    InvalidSignature = 6101,
    
    #[msg("Gateway not configured")]
    GatewayNotConfigured = 6102,
    
    // Origin system errors (6200-6299)
    #[msg("Origin PDA not found")]
    OriginPdaNotFound = 6200,
    
    #[msg("Origin PDA already exists")]
    OriginPdaAlreadyExists = 6201,
    
    #[msg("Invalid origin PDA derivation")]
    InvalidOriginPdaDerivation = 6202,
    
    #[msg("Unauthorized origin access")]
    UnauthorizedOriginAccess = 6203,
    
    #[msg("Origin token ID mismatch")]
    OriginTokenIdMismatch = 6204,
    
    #[msg("Unsupported origin chain")]
    UnsupportedOriginChain = 6205,
    
    #[msg("Invalid origin metadata")]
    InvalidOriginMetadata = 6206,
    
    #[msg("Invalid origin timestamp")]
    InvalidOriginTimestamp = 6207,
    
    #[msg("Origin data corruption detected")]
    OriginDataCorruption = 6208,
    
    #[msg("Invalid TSS signature with origin context")]
    InvalidOriginTssSignature = 6209,
    
    #[msg("Cross-chain message integrity failure")]
    MessageIntegrityFailure = 6210,
    
    #[msg("Inconsistent origin data")]
    InconsistentOriginData = 6211,
    
    // Chain compatibility errors (6300-6399)
    #[msg("Unsupported chain combination")]
    UnsupportedChainCombination = 6300,
    
    #[msg("Invalid destination chain")]
    InvalidDestinationChain = 6301,
    
    #[msg("Unsupported chain")]
    UnsupportedChain = 6302,
    
    #[msg("Invalid recipient address format")]
    InvalidRecipientAddress = 6303,
    
    // System errors (6400-6499)
    #[msg("Nonce overflow")]
    NonceOverflow = 6400,
    
    #[msg("Invalid metadata URI")]
    InvalidMetadataUri = 6401,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp = 6402,
}
```

### Error Handling Examples

```typescript
// Comprehensive error handling
async function handleUniversalNftOperation(operation: () => Promise<string>): Promise<string> {
  try {
    return await operation();
  } catch (error: any) {
    switch (error.code) {
      case 6000:
        throw new Error("Invalid token ID - check token ID generation");
      case 6001:
        throw new Error("Invalid authority - ensure correct signer");
      case 6002:
        throw new Error("Invalid chain ID - check supported chains");
      case 6003:
        throw new Error("Invalid message format - check cross-chain message encoding");
      case 6004:
        throw new Error("Invalid nonce - check replay protection");
      case 6100:
        throw new Error("Unauthorized gateway - only ZetaChain gateway can call");
      case 6101:
        throw new Error("Invalid TSS signature - check signature verification");
      case 6200:
        throw new Error("Origin PDA not found - NFT may not exist");
      case 6201:
        throw new Error("Origin PDA already exists - duplicate token ID");
      case 6202:
        throw new Error("Invalid origin PDA derivation - check seeds");
      case 6300:
        throw new Error("Unsupported chain combination - check chain compatibility");
      default:
        throw new Error(`Unknown error: ${error.message} (code: ${error.code})`);
    }
  }
}

// Usage example
try {
  const tx = await handleUniversalNftOperation(() =>
    program.methods.mintNft(name, symbol, uri).rpc()
  );
  console.log("Success:", tx);
} catch (error) {
  console.error("Operation failed:", error.message);
}
```

## Events

The Universal NFT program emits comprehensive events for tracking the complete NFT lifecycle and origin system operations.

### Core Origin Events

```rust
#[event]
pub struct NftOriginCreated {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub collection: Pubkey,
    pub origin_chain: u64,
    pub metadata_uri: String,
    pub created_at: i64,
    pub is_solana_native: bool,
}

#[event]
pub struct NftReturningToSolana {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub new_mint: Pubkey,
    pub collection: Pubkey,
    pub origin_chain: u64,
    pub metadata_preserved: bool,
    pub cycle_count: u64,
    pub returned_at: i64,
}

#[event]
pub struct CrossChainCycleCompleted {
    pub token_id: u64,
    pub origin_chain: u64,
    pub destination_chain: u64,
    pub cycle_count: u64,
    pub total_chains_visited: u64,
    pub cycle_duration: i64,
    pub completed_at: i64,
}
```

### Enhanced Core Events

```rust
#[event]
pub struct TokenMintedWithOrigin {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub uri: String,
    pub origin_chain: u64,
    pub is_solana_native: bool,
    pub original_mint: Pubkey,
    pub created_at: i64,
}

#[event]
pub struct TokenTransferWithOrigin {
    pub collection: Pubkey,
    pub token_id: u64,
    pub from: Pubkey,
    pub destination_chain_id: u64,
    pub recipient: Vec<u8>,
    pub uri: String,
    pub origin_chain: u64,
    pub original_mint: Pubkey,
    pub is_returning_to_origin: bool,
    pub cycle_count: u64,
    pub transferred_at: i64,
}

#[event]
pub struct TokenTransferReceivedWithOrigin {
    pub collection: Pubkey,
    pub token_id: u64,
    pub source_chain_id: u64,
    pub sender: [u8; 20],
    pub recipient: Pubkey,
    pub uri: String,
    pub origin_chain: u64,
    pub original_mint: Pubkey,
    pub is_returning: bool,
    pub is_returning_to_origin: bool,
    pub cycle_count: u64,
    pub origin_preserved: bool,
    pub scenario: String,
    pub received_at: i64,
}
```

### Event Listening Examples

```typescript
// Listen to origin system events
program.addEventListener('NftOriginCreated', (event) => {
  console.log(`New origin created: Token ID ${event.tokenId}, Chain ${event.originChain}`);
  console.log(`Metadata: ${event.metadataUri}`);
  console.log(`Is Solana native: ${event.isSolanaNative}`);
});

program.addEventListener('NftReturningToSolana', (event) => {
  console.log(`NFT returning: Token ID ${event.tokenId}, Cycle ${event.cycleCount}`);
  console.log(`Original mint: ${event.originalMint.toBase58()}`);
  console.log(`New mint: ${event.newMint.toBase58()}`);
  console.log(`Metadata preserved: ${event.metadataPreserved}`);
});

program.addEventListener('TokenTransferWithOrigin', (event) => {
  console.log(`Cross-chain transfer: Token ID ${event.tokenId}`);
  console.log(`From: ${event.from.toBase58()}`);
  console.log(`To chain: ${event.destinationChainId}`);
  console.log(`Origin chain: ${event.originChain}`);
  console.log(`Returning to origin: ${event.isReturningToOrigin}`);
});

program.addEventListener('TokenTransferReceivedWithOrigin', (event) => {
  console.log(`NFT received: Token ID ${event.tokenId}`);
  console.log(`From chain: ${event.sourceChainId}`);
  console.log(`Scenario: ${event.scenario}`);
  console.log(`Origin preserved: ${event.originPreserved}`);
});

// Event filtering and analysis
async function analyzeNftJourney(program: Program, tokenId: number) {
  const events = await program.account.eventQueue.all(); // Hypothetical event queue
  
  const nftEvents = events.filter(event => 
    event.account.data.tokenId === tokenId
  );

  const journey = {
    tokenId,
    originChain: 0,
    currentChain: 103, // Solana
    chainsVisited: new Set<number>(),
    transferCount: 0,
    cycleCount: 0,
    events: nftEvents,
  };

  for (const event of nftEvents) {
    switch (event.name) {
      case 'NftOriginCreated':
        journey.originChain = event.data.originChain;
        journey.chainsVisited.add(event.data.originChain);
        break;
      case 'TokenTransferWithOrigin':
        journey.transferCount++;
        journey.chainsVisited.add(event.data.destinationChainId);
        if (event.data.isReturningToOrigin) {
          journey.cycleCount++;
        }
        break;
      case 'TokenTransferReceivedWithOrigin':
        journey.currentChain = 103; // Back on Solana
        break;
    }
  }

  return {
    ...journey,
    chainsVisited: Array.from(journey.chainsVisited),
  };
}
```

## Security Considerations

### Access Control

#### Collection Authority
- Only the designated authority can modify collection settings
- Authority can set universal addresses and configure connected contracts
- Authority controls origin system parameters

#### Gateway-Only Functions
- Critical functions like `on_call` and `on_revert` restricted to authorized ZetaChain gateway
- Gateway program ID validation: `ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis`
- Gateway PDA validation for additional security

#### Origin PDA Protection
- Origin PDAs can only be created/modified through authorized program instructions
- Strict validation of origin data integrity
- Protection against unauthorized origin data tampering

### Cryptographic Security

#### Enhanced TSS Signatures
```rust
pub fn verify_enhanced_tss_signature(
    sender: &[u8; 20],
    message: &EnhancedCrossChainMessage,
    signature: &[u8; 65],
    tss_address: &[u8; 20],
) -> Result<()> {
    // Create comprehensive message hash including origin data
    let mut hasher = keccak::Hasher::default();
    hasher.hash(sender);
    hasher.hash(&message.token_id.to_le_bytes());
    hasher.hash(message.metadata_uri.as_bytes());
    hasher.hash(&message.recipient);
    hasher.hash(&message.origin_chain.to_le_bytes());
    hasher.hash(message.original_mint.as_ref());
    hasher.hash(&[if message.is_solana_native { 1 } else { 0 }]);
    hasher.hash(&message.created_at.to_le_bytes());
    
    let message_hash = hasher.result();
    
    // Verify ECDSA signature against TSS public key
    let recovered = recover_ecdsa_public_key(&message_hash.0, signature)?;
    let recovered_address = pubkey_to_eth_address(&recovered);
    
    require!(
        recovered_address == *tss_address,
        UniversalNftError::InvalidOriginTssSignature
    );
    
    Ok(())
}
```

#### Token ID Security
```rust
// Deterministic token ID generation with collision resistance
fn generate_secure_token_id(
    mint: &Pubkey,
    block_number: u64,
    sequence: u64,
    collection: &Pubkey,
) -> Result<u64> {
    let mut hasher = keccak::Hasher::default();
    hasher.hash(mint.as_ref());                    // 32 bytes - unique mint
    hasher.hash(&block_number.to_le_bytes());      // 8 bytes - block number
    hasher.hash(&sequence.to_le_bytes());          // 8 bytes - sequence
    hasher.hash(collection.as_ref());              // 32 bytes - collection
    hasher.hash(&Clock::get()?.unix_timestamp.to_le_bytes()); // 8 bytes - timestamp
    
    let hash = hasher.result();
    let token_id = u64::from_le_bytes(hash.0[0..8].try_into().unwrap());
    
    // Ensure token ID is not zero
    Ok(if token_id == 0 { 1 } else { token_id })
}
```

#### Message Integrity
```rust
pub fn validate_message_integrity_with_origin(
    message: &EnhancedCrossChainMessage,
    expected_hash: &[u8; 32],
) -> Result<()> {
    let mut hasher = keccak::Hasher::default();
    hasher.hash(&message.token_id.to_le_bytes());
    hasher.hash(message.metadata_uri.as_bytes());
    hasher.hash(&message.recipient);
    hasher.hash(&message.sender);
    hasher.hash(&message.origin_chain.to_le_bytes());
    hasher.hash(message.original_mint.as_ref());
    hasher.hash(&[if message.is_solana_native { 1 } else { 0 }]);
    hasher.hash(&message.created_at.to_le_bytes());
    hasher.hash(&message.cycle_count.to_le_bytes());
    
    let calculated_hash = hasher.result();
    
    require!(
        calculated_hash.0 == *expected_hash,
        UniversalNftError::MessageIntegrityFailure
    );
    
    Ok(())
}
```

### Economic Security

#### Rent Exemption
- All accounts including origin PDAs are rent-exempt to prevent unexpected closure
- Collection account: ~0.00203 SOL
- Origin PDA: ~0.00185 SOL per NFT
- Total per NFT: ~0.00388 SOL

#### Compute Budget Management
```typescript
// Add compute budget for origin system operations
const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
  units: 350_000, // Increased for origin system
});

const computePriceIx = ComputeBudgetProgram.setComputeUnitPrice({
  microLamports: 1000, // Adjust based on network conditions
});

transaction.add(computeBudgetIx, computePriceIx);
```

### Operational Security

#### Origin Data Validation
```rust
impl NftOrigin {
    pub fn validate_origin_integrity(&self) -> Result<()> {
        // Validate token ID is not zero
        require!(
            self.token_id > 0,
            UniversalNftError::InvalidOriginTokenId
        );
        
        // Validate origin chain is supported
        require!(
            is_supported_chain(self.chain_of_origin),
            UniversalNftError::UnsupportedOriginChain
        );
        
        // Validate metadata URI format
        require!(
            !self.metadata_uri.is_empty() && self.metadata_uri.len() <= 200,
            UniversalNftError::InvalidOriginMetadata
        );
        
        // Validate creation timestamp
        require!(
            self.created_at > 0,
            UniversalNftError::InvalidOriginTimestamp
        );
        
        Ok(())
    }
}
```

#### Replay Protection
```rust
impl Collection {
    pub fn increment_nonce_with_origin(&mut self, token_id: u64) -> Result<u64> {
        let current_nonce = self.nonce;
        
        require!(
            current_nonce < u64::MAX,
            UniversalNftError::NonceOverflow
        );
        
        self.nonce = current_nonce.checked_add(1)
            .ok_or(UniversalNftError::NonceOverflow)?;
        
        emit!(NonceUpdatedWithOrigin {
            collection: self.key(),
            old_nonce: current_nonce,
            new_nonce: self.nonce,
            token_id,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(current_nonce)
    }
}
```

## Best Practices

### Development Best Practices

#### Account Validation
```typescript
// Always validate accounts before operations
async function validateCollection(
  program: Program,
  collection: PublicKey
): Promise<void> {
  const accountInfo = await program.provider.connection.getAccountInfo(collection);
  if (!accountInfo) {
    throw new Error("Collection does not exist");
  }
  
  if (!accountInfo.owner.equals(program.programId)) {
    throw new Error("Invalid collection owner");
  }
  
  // Validate collection data
  const collectionData = await program.account.collection.fetch(collection);
  if (!collectionData.authority) {
    throw new Error("Invalid collection authority");
  }
}

async function validateOriginPda(
  program: Program,
  originPda: PublicKey,
  expectedTokenId: number
): Promise<void> {
  try {
    const originData = await program.account.nftOrigin.fetch(originPda);
    
    if (originData.tokenId.toNumber() !== expectedTokenId) {
      throw new Error(`Token ID mismatch: expected ${expectedTokenId}, got ${originData.tokenId}`);
    }
    
    // Validate origin data integrity
    if (!originData.metadataUri || originData.metadataUri.length === 0) {
      throw new Error("Invalid metadata URI");
    }
    
    if (originData.createdAt.toNumber() <= 0) {
      throw new Error("Invalid creation timestamp");
    }
  } catch (error) {
    throw new Error(`Origin validation failed: ${error.message}`);
  }
}
```

#### Error Handling
```typescript
// Comprehensive error handling with retry logic
async function executeWithRetry<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  delay: number = 1000
): Promise<T> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error: any) {
      console.error(`Attempt ${attempt} failed:`, error.message);
      
      // Don't retry on certain errors
      if (error.code === 6001 || error.code === 6002) {
        throw error;
      }
      
      if (attempt === maxRetries) {
        throw new Error(`Operation failed after ${maxRetries} attempts: ${error.message}`);
      }
      
      // Wait before retry
      await new Promise(resolve => setTimeout(resolve, delay * attempt));
    }
  }
  
  throw new Error("Unexpected error in retry logic");
}

// Usage
const tx = await executeWithRetry(() =>
  program.methods.mintNft(name, symbol, uri).rpc()
);
```

#### Transaction Building
```typescript
// Build transactions with proper compute budget and priority fees
async function buildOptimizedTransaction(
  connection: Connection,
  instructions: TransactionInstruction[],
  payer: PublicKey,
  priorityFee: number = 1000
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // Add compute budget instructions
  transaction.add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 350_000, // Increased for origin system
    })
  );
  
  transaction.add(
    ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: priorityFee,
    })
  );
  
  // Add main instructions
  transaction.add(...instructions);
  
  // Get recent blockhash
  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = payer;
  
  return transaction;
}
```

### Production Deployment

#### Environment Configuration
```typescript
// Environment-specific configuration
interface NetworkConfig {
  rpcUrl: string;
  programId: PublicKey;
  gatewayId: PublicKey;
  commitment: Commitment;
  confirmTimeout: number;
}

const NETWORK_CONFIGS: Record<string, NetworkConfig> = {
  'mainnet-beta': {
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    programId: new PublicKey('GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip'),
    gatewayId: new PublicKey('ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis'),
    commitment: 'confirmed',
    confirmTimeout: 60000,
  },
  'devnet': {
    rpcUrl: 'https://api.devnet.solana.com',
    programId: new PublicKey('GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip'),
    gatewayId: new PublicKey('ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis'),
    commitment: 'confirmed',
    confirmTimeout: 30000,
  },
  'localnet': {
    rpcUrl: 'http://127.0.0.1:8899',
    programId: new PublicKey('GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip'),
    gatewayId: new PublicKey('ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis'),
    commitment: 'processed',
    confirmTimeout: 10000,
  },
};

function getNetworkConfig(network: string): NetworkConfig {
  const config = NETWORK_CONFIGS[network];
  if (!config) {
    throw new Error(`Unsupported network: ${network}`);
  }
  return config;
}
```

#### Monitoring and Health Checks
```typescript
// Health monitoring for production deployment
class UniversalNftHealthMonitor {
  constructor(
    private connection: Connection,
    private program: Program,
    private collectionPda: PublicKey
  ) {}

  async checkSystemHealth(): Promise<HealthReport> {
    const report: HealthReport = {
      timestamp: Date.now(),
      status: 'healthy',
      checks: [],
    };

    // Check 1: Program account exists and is executable
    try {
      const programAccount = await this.connection.getAccountInfo(this.program.programId);
      if (!programAccount || !programAccount.executable) {
        report.checks.push({
          name: 'Program Account',
          status: 'error',
          message: 'Program account not found or not executable',
        });
        report.status = 'unhealthy';
      } else {
        report.checks.push({
          name: 'Program Account',
          status: 'ok',
          message: 'Program account is healthy',
        });
      }
    } catch (error) {
      report.checks.push({
        name: 'Program Account',
        status: 'error',
        message: `Failed to check program account: ${error.message}`,
      });
      report.status = 'unhealthy';
    }

    // Check 2: Collection account integrity
    try {
      const collectionData = await this.program.account.collection.fetch(this.collectionPda);
      const stats = collectionData.getStats();
      
      if (stats[0] < stats[1]) { // total < native
        report.checks.push({
          name: 'Collection Statistics',
          status: 'error',
          message: `Invalid stats: total(${stats[0]}) < native(${stats[1]})`,
        });
        report.status = 'degraded';
      } else {
        report.checks.push({
          name: 'Collection Statistics',
          status: 'ok',
          message: `Total: ${stats[0]}, Native: ${stats[1]}, Cross-chain: ${stats[2]}`,
        });
      }
    } catch (error) {
      report.checks.push({
        name: 'Collection Statistics',
        status: 'error',
        message: `Failed to fetch collection data: ${error.message}`,
      });
      report.status = 'unhealthy';
    }

    // Check 3: RPC connection health
    try {
      const slot = await this.connection.getSlot();
      const health = await this.connection.getHealth();
      
      if (health !== 'ok') {
        report.checks.push({
          name: 'RPC Health',
          status: 'warning',
          message: `RPC health: ${health}`,
        });
        report.status = 'degraded';
      } else {
        report.checks.push({
          name: 'RPC Health',
          status: 'ok',
          message: `RPC healthy, current slot: ${slot}`,
        });
      }
    } catch (error) {
      report.checks.push({
        name: 'RPC Health',
        status: 'error',
        message: `RPC connection failed: ${error.message}`,
      });
      report.status = 'unhealthy';
    }

    return report;
  }

  async monitorOriginSystemIntegrity(): Promise<OriginIntegrityReport> {
    // Sample a few origin PDAs to check integrity
    const collectionData = await this.program.account.collection.fetch(this.collectionPda);
    const totalMinted = collectionData.totalMinted.toNumber();
    
    const sampleSize = Math.min(10, totalMinted);
    const integrityReport: OriginIntegrityReport = {
      totalChecked: sampleSize,
      validOrigins: 0,
      invalidOrigins: 0,
      errors: [],
    };

    for (let i = 0; i < sampleSize; i++) {
      const tokenId = Math.floor(Math.random() * totalMinted) + 1;
      
      try {
        const [originPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("nft_origin"),
            new BN(tokenId).toArrayLike(Buffer, 'le', 8)
          ],
          this.program.programId
        );

        const originData = await this.program.account.nftOrigin.fetch(originPda);
        
        // Validate origin data
        if (this.validateOriginData(originData)) {
          integrityReport.validOrigins++;
        } else {
          integrityReport.invalidOrigins++;
          integrityReport.errors.push(`Invalid origin data for token ${tokenId}`);
        }
      } catch (error) {
        integrityReport.invalidOrigins++;
        integrityReport.errors.push(`Failed to check token ${tokenId}: ${error.message}`);
      }
    }

    return integrityReport;
  }

  private validateOriginData(origin: any): boolean {
    return (
      origin.tokenId.toNumber() > 0 &&
      origin.metadataUri.length > 0 &&
      origin.createdAt.toNumber() > 0 &&
      origin.originalMint &&
      origin.collection
    );
  }
}

interface HealthReport {
  timestamp: number;
  status: 'healthy' | 'degraded' | 'unhealthy';
  checks: Array<{
    name: string;
    status: 'ok' | 'warning' | 'error';
    message: string;
  }>;
}

interface OriginIntegrityReport {
  totalChecked: number;
  validOrigins: number;
  invalidOrigins: number;
  errors: string[];
}
```

### Performance Optimization

#### Batch Operations
```typescript
// Batch multiple operations for efficiency
async function batchMintNfts(
  program: Program,
  collectionPda: PublicKey,
  authority: Keypair,
  nftData: Array<{ name: string; symbol: string; uri: string; recipient: PublicKey }>
): Promise<string[]> {
  const transactions: Transaction[] = [];
  const signatures: string[] = [];

  // Group operations into batches to stay within transaction size limits
  const batchSize = 3; // Conservative batch size for NFT minting
  
  for (let i = 0; i < nftData.length; i += batchSize) {
    const batch = nftData.slice(i, i + batchSize);
    const transaction = new Transaction();
    
    // Add compute budget
    transaction.add(
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })
    );

    const signers: Keypair[] = [authority];

    for (const nft of batch) {
      const nftMint = Keypair.generate();
      signers.push(nftMint);

      // Get next token ID (this would need to be tracked properly)
      const collectionData = await program.account.collection.fetch(collectionPda);
      const nextTokenId = collectionData.nextTokenId.add(new BN(signers.length - 2));

      const instruction = await program.methods
        .mintNft(nft.name, nft.symbol, nft.uri)
        .accounts({
          collection: collectionPda,
          authority: authority.publicKey,
          nftMint: nftMint.publicKey,
          recipient: nft.recipient,
          // ... other accounts
        })
        .instruction();

      transaction.add(instruction);
    }

    // Send batch transaction
    const signature = await program.provider.sendAndConfirm(transaction, signers);
    signatures.push(signature);
  }

  return signatures;
}
```

#### Caching and Optimization
```typescript
// Cache frequently accessed data
class UniversalNftCache {
  private collectionCache = new Map<string, any>();
  private originCache = new Map<string, any>();
  private cacheTimeout = 60000; // 1 minute

  async getCollection(
    program: Program,
    collectionPda: PublicKey
  ): Promise<any> {
    const key = collectionPda.toBase58();
    const cached = this.collectionCache.get(key);
    
    if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
      return cached.data;
    }

    const data = await program.account.collection.fetch(collectionPda);
    this.collectionCache.set(key, {
      data,
      timestamp: Date.now(),
    });

    return data;
  }

  async getOrigin(
    program: Program,
    originPda: PublicKey
  ): Promise<any> {
    const key = originPda.toBase58();
    const cached = this.originCache.get(key);
    
    if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
      return cached.data;
    }

    try {
      const data = await program.account.nftOrigin.fetch(originPda);
      this.originCache.set(key, {
        data,
        timestamp: Date.now(),
      });
      return data;
    } catch (error) {
      // Cache null result for failed fetches (shorter timeout)
      this.originCache.set(key, {
        data: null,
        timestamp: Date.now(),
      });
      throw error;
    }
  }

  clearCache(): void {
    this.collectionCache.clear();
    this.originCache.clear();
  }
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Origin PDA Derivation Issues
**Problem**: Origin PDA derivation fails or produces unexpected addresses

**Solution**:
```typescript
// Ensure correct token ID format and seeds
function deriveOriginPda(tokenId: number, programId: PublicKey): [PublicKey, number] {
  // Use little-endian encoding for token ID
  const tokenIdBuffer = new BN(tokenId).toArrayLike(Buffer, 'le', 8);
  
  const [originPda, bump] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("nft_origin"),
      tokenIdBuffer
    ],
    programId
  );
  
  return [originPda, bump];
}

// Validate derivation
async function validateOriginPdaDerivation(
  program: Program,
  tokenId: number
): Promise<boolean> {
  const [expectedPda] = deriveOriginPda(tokenId, program.programId);
  
  try {
    const originData = await program.account.nftOrigin.fetch(expectedPda);
    return originData.tokenId.toNumber() === tokenId;
  } catch (error) {
    console.error(`Origin PDA validation failed for token ${tokenId}:`, error);
    return false;
  }
}
```

#### 2. Two-Scenario Logic Errors
**Problem**: Incorrect scenario detection or handling

**Solution**:
```typescript
// Properly check origin PDA existence
async function determineScenario(
  program: Program,
  tokenId: number
): Promise<'A' | 'B'> {
  const [originPda] = deriveOriginPda(tokenId, program.programId);
  
  try {
    const originData = await program.account.nftOrigin.fetch(originPda);
    console.log("Scenario A: NFT returning to Solana");
    console.log(`Original mint: ${originData.originalMint.toBase58()}`);
    console.log(`Original metadata: ${originData.metadataUri}`);
    return 'A';
  } catch (error) {
    if (error.message.includes('Account does not exist')) {
      console.log("Scenario B: NFT first time on Solana");
      return 'B';
    } else {
      throw error;
    }
  }
}

// Use init_if_needed constraint in Anchor
#[account(
    init_if_needed,
    payer = payer,
    space = 8 + NftOrigin::INIT_SPACE,
    seeds = [b"nft_origin", &token_id.to_le_bytes()],
    bump
)]
pub nft_origin: Account<'info, NftOrigin>,
```

#### 3. Cross-Chain Message Format Issues
**Problem**: Message decoding failures between chains

**Solution**:
```typescript
// Robust message decoding with fallback
function decodeCrossChainMessage(messageData: Buffer): EnhancedCrossChainMessage {
  // Try ABI decoding first (from EVM)
  try {
    return decodeAbiMessage(messageData);
  } catch (abiError) {
    console.warn("ABI decoding failed:", abiError.message);
    
    // Fall back to Borsh decoding (from Solana)
    try {
      return EnhancedCrossChainMessage.deserialize(messageData);
    } catch (borshError) {
      console.error("Borsh decoding failed:", borshError.message);
      throw new Error(`Failed to decode message: ABI(${abiError.message}), Borsh(${borshError.message})`);
    }
  }
}

// Validate message format before processing
function validateCrossChainMessage(message: EnhancedCrossChainMessage): void {
  if (!message.tokenId || message.tokenId <= 0) {
    throw new Error("Invalid token ID");
  }
  
  if (!message.metadataUri || message.metadataUri.length === 0) {
    throw new Error("Invalid metadata URI");
  }
  
  if (!message.recipient || message.recipient.length === 0) {
    throw new Error("Invalid recipient");
  }
  
  if (!isValidChainId(message.originChain)) {
    throw new Error(`Invalid origin chain: ${message.originChain}`);
  }
}
```

#### 4. Performance and Compute Budget Issues
**Problem**: Transactions fail due to compute budget exceeded

**Solution**:
```typescript
// Optimize compute usage for origin system
async function optimizedMintNft(
  program: Program,
  collectionPda: PublicKey,
  authority: Keypair,
  name: string,
  symbol: string,
  uri: string,
  recipient: PublicKey
): Promise<string> {
  // Pre-calculate all PDAs to reduce compute
  const collectionData = await program.account.collection.fetch(collectionPda);
  const nextTokenId = collectionData.nextTokenId;
  
  const [originPda, originBump] = deriveOriginPda(nextTokenId.toNumber(), program.programId);
  
  const nftMint = Keypair.generate();
  
  // Build transaction with optimized compute budget
  const transaction = new Transaction();
  
  // Add compute budget instructions
  transaction.add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 350_000, // Sufficient for origin system operations
    })
  );
  
  transaction.add(
    ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1000, // Adjust based on network conditions
    })
  );
  
  // Add mint instruction
  const mintInstruction = await program.methods
    .mintNft(name, symbol, uri)
    .accounts({
      collection: collectionPda,
      authority: authority.publicKey,
      nftMint: nftMint.publicKey,
      recipient: recipient,
      nftOrigin: originPda,
      // ... other accounts
    })
    .instruction();
  
  transaction.add(mintInstruction);
  
  // Send transaction
  const signature = await program.provider.sendAndConfirm(
    transaction,
    [authority, nftMint],
    {
      commitment: 'confirmed',
      maxRetries: 3,
    }
  );
  
  return signature;
}
```

#### 5. Gateway Integration Issues
**Problem**: Gateway calls fail with origin data

**Solution**:
```typescript
// Verify gateway configuration and access
async function validateGatewayIntegration(
  connection: Connection,
  program: Program,
  collectionPda: PublicKey
): Promise<void> {
  // Check gateway program exists
  const gatewayProgramId = new PublicKey("ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis");
  const gatewayAccount = await connection.getAccountInfo(gatewayProgramId);
  
  if (!gatewayAccount || !gatewayAccount.executable) {
    throw new Error("Gateway program not found or not executable");
  }
  
  // Verify gateway PDA derivation
  const [gatewayPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("meta")],
    gatewayProgramId
  );
  
  const gatewayPdaAccount = await connection.getAccountInfo(gatewayPda);
  if (!gatewayPdaAccount) {
    throw new Error("Gateway PDA not found");
  }
  
  // Check collection gateway configuration
  const collectionData = await program.account.collection.fetch(collectionPda);
  if (!collectionData.gatewayAddress) {
    console.warn("Gateway address not set in collection");
  }
  
  console.log("Gateway integration validated successfully");
}

// Test gateway call with proper error handling
async function testGatewayCall(
  program: Program,
  collectionPda: PublicKey,
  testMessage: Buffer
): Promise<void> {
  try {
    const senderAddress = new Uint8Array(20).fill(1);
    const sourceChainId = 11155111; // Ethereum Sepolia
    const nonce = 1;
    
    // This would typically be called by the gateway
    const tx = await program.methods
      .onCall(
        Array.from(senderAddress),
        new BN(sourceChainId),
        Array.from(testMessage),
        new BN(nonce)
      )
      .accounts({
        collection: collectionPda,
        gateway: new PublicKey("ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis"),
        // ... other accounts
      })
      .rpc();
    
    console.log("Gateway call test successful:", tx);
  } catch (error) {
    if (error.code === 6100) {
      console.error("Unauthorized gateway access - check gateway program ID");
    } else if (error.code === 6101) {
      console.error("Invalid TSS signature - check signature verification");
    } else {
      console.error("Gateway call failed:", error.message);
    }
    throw error;
  }
}
```

### Debug Commands and Tools

```bash
# Check program account
solana account GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip --output json

# Monitor program logs
solana logs GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip

# Check specific transaction
solana confirm <TRANSACTION_SIGNATURE> -v

# Verify origin PDA derivation
node -e "
const { PublicKey } = require('@solana/web3.js');
const BN = require('bn.js');
const tokenId = 12345;
const [pda] = PublicKey.findProgramAddressSync(
  [Buffer.from('nft_origin'), new BN(tokenId).toArrayLike(Buffer, 'le', 8)],
  new PublicKey('GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip')
);
console.log('Origin PDA:', pda.toBase58());
"

# Check collection statistics
solana account <COLLECTION_PDA_ADDRESS> --output json | jq '.account.data'
```

### Support and Resources

- **Program Explorer**: https://explorer.solana.com/address/GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip?cluster=devnet
- **Anchor Documentation**: https://www.anchor-lang.com/
- **Solana Documentation**: https://docs.solana.com/
- **ZetaChain Documentation**: https://docs.zetachain.com/
- **GitHub Repository**: https://github.com/zeta-chain/standard-contracts

For additional support, please refer to the [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) guide or reach out to the development team through the official channels.

---

*This API documentation covers the complete Universal NFT program with NFT Origin System. For the latest updates and additional examples, please refer to the GitHub repository and official documentation.*