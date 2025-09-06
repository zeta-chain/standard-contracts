# Universal NFT Program - API Reference

## Program Information

- **Program ID**: `6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke`
- **Network**: Devnet (deployed), ready for mainnet
- **Framework**: Anchor 0.30.1
- **Language**: Rust

## Instructions

### `initialize_collection`

Initialize a new Universal NFT collection with cross-chain capabilities.

**Parameters:**
- `name: String` - Collection name (max 64 chars)
- `symbol: String` - Collection symbol (max 10 chars) 
- `uri: String` - Collection metadata URI
- `tss_address: [u8; 20]` - ZetaChain TSS address for gateway integration

**Accounts:**
- `authority: Signer` - Collection authority (mutable, payer)
- `collection: Account<Collection>` - Collection PDA (init)
- `system_program: Program<System>`
- `rent: Sysvar<Rent>`

**PDA Seeds:** `[b"collection", authority.key(), name.as_bytes()]`

**Events Emitted:**
- `CollectionInitialized`

**Example:**
```typescript
await program.methods
  .initializeCollection(
    "My Universal Collection",
    "MUC", 
    "https://example.com/collection.json",
    Array.from(tssAddress)
  )
  .accounts({
    authority: authority.publicKey,
    collection: collectionPda,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([authority])
  .rpc();
```

### `mint_nft`

Mint a new NFT with Origin tracking system.

**Parameters:**
- `name: String` - NFT name
- `symbol: String` - NFT symbol
- `uri: String` - NFT metadata URI

**Accounts:**
- `authority: Signer` - Mint authority (mutable)
- `collection: Account<Collection>` - Collection account (mutable)
- `nft_mint: Signer` - New NFT mint keypair
- `nft_token_account: Account<TokenAccount>` - Associated token account (init)
- `recipient: UncheckedAccount` - NFT recipient
- `nft_origin: Account<NftOrigin>` - Origin PDA (init)
- `nft_metadata: UncheckedAccount` - Metaplex metadata account
- `master_edition: UncheckedAccount` - Metaplex master edition account
- Token and system programs

**PDA Seeds:**
- Origin: `[b"nft_origin", collection.next_token_id.to_le_bytes()]`
- Metadata: `[b"metadata", TOKEN_METADATA_PROGRAM_ID, nft_mint.key()]`
- Master Edition: `[b"metadata", TOKEN_METADATA_PROGRAM_ID, nft_mint.key(), b"edition"]`

**Events Emitted:**
- `TokenMinted`
- `NftOriginCreated`

**Example:**
```typescript
const nftMint = Keypair.generate();
const [nftOrigin] = PublicKey.findProgramAddressSync(
  [Buffer.from("nft_origin"), nextTokenId.toArrayLike(Buffer, 'le', 8)],
  program.programId
);

await program.methods
  .mintNft("My NFT #1", "MN1", "https://example.com/nft1.json")
  .accounts({
    authority: authority.publicKey,
    collection: collectionPda,
    nftMint: nftMint.publicKey,
    nftTokenAccount: tokenAccountPda,
    recipient: recipient.publicKey,
    nftOrigin,
    // ... other accounts
  })
  .signers([authority, nftMint])
  .rpc();
```

### `transfer_cross_chain`

Transfer NFT to another blockchain via ZetaChain gateway.

**Parameters:**
- `destination_chain: u64` - Target chain ID (1=Ethereum, 56=BSC, etc.)
- `recipient: Vec<u8>` - Recipient address on destination chain

**Accounts:**
- `authority: Signer` - Transfer authority
- `collection: Account<Collection>` - Collection account
- `nft_mint: Account<Mint>` - NFT mint account
- `nft_token_account: Account<TokenAccount>` - Source token account (mutable)
- `nft_origin: Account<NftOrigin>` - Origin PDA
- `gateway: UncheckedAccount` - ZetaChain gateway account
- Token programs

**Events Emitted:**
- `TokenTransferInitiated`

**Example:**
```typescript
await program.methods
  .transferCrossChain(
    1, // Ethereum
    Array.from(Buffer.from("0x742d35Cc6634C0532925a3b8D82C3E4F4", "hex"))
  )
  .accounts({
    authority: authority.publicKey,
    collection: collectionPda,
    nftMint: nftMint.publicKey,
    nftTokenAccount: tokenAccountPda,
    nftOrigin: originPda,
    gateway: gatewayPda,
    // ... other accounts
  })
  .signers([authority])
  .rpc();
```

### `on_call`

Handle incoming cross-chain NFT transfers from ZetaChain gateway.

**Parameters:**
- `sender: Vec<u8>` - Sender address from source chain
- `source_chain: u64` - Source chain ID
- `message: Vec<u8>` - Cross-chain message data
- `nonce: u64` - Message nonce for replay protection

**Message Format:**
```
[token_id: 8 bytes][uri_length: 4 bytes][uri: variable][recipient: 32 bytes]
```

**Accounts:**
- `collection: Account<Collection>` - Collection account (mutable)
- `gateway: UncheckedAccount` - ZetaChain gateway account
- `connected: Account<Connected>` - Connected contract PDA
- Various mint and token accounts (conditional)

**Events Emitted:**
- `TokenTransferReceived`
- `NftOriginCreated` (for new NFTs)

### `set_universal`

Set the universal contract address for the collection.

**Parameters:**
- `universal_address: Vec<u8>` - Universal contract address

**Accounts:**
- `authority: Signer` - Collection authority
- `collection: Account<Collection>` - Collection account (mutable)

**Events Emitted:**
- `SetUniversal`

### `set_connected`

Set connected contract address for a specific chain.

**Parameters:**
- `chain_id: Vec<u8>` - Target chain ID
- `contract_address: Vec<u8>` - Contract address on target chain

**Accounts:**
- `authority: Signer` - Collection authority (mutable, payer)
- `collection: Account<Collection>` - Collection account
- `connected: Account<Connected>` - Connected contract PDA (init)

**PDA Seeds:** `[b"connected", collection.key(), chain_id]`

**Events Emitted:**
- `SetConnected`

### `on_revert`

Handle failed cross-chain transfers by minting NFT back to original sender.

**Parameters:**
- `sender: Vec<u8>` - Original sender address
- `source_chain: u64` - Source chain ID
- `message: Vec<u8>` - Original message data
- `nonce: u64` - Message nonce

**Events Emitted:**
- `TokenTransferReverted`

## Account Structures

### `Collection`

```rust
pub struct Collection {
    pub authority: Pubkey,           // Collection authority
    pub name: String,                // Collection name
    pub symbol: String,              // Collection symbol
    pub uri: String,                 // Collection metadata URI
    pub next_token_id: u64,          // Next token ID to mint
    pub total_minted: u64,           // Total NFTs minted
    pub solana_native_count: u64,    // Count of Solana-native NFTs
    pub cross_chain_count: u64,      // Count of cross-chain NFTs
    pub universal_address: Vec<u8>,  // Universal contract address
    pub tss_address: [u8; 20],       // TSS address for gateway
    pub nonce: u64,                  // Message nonce counter
    pub bump: u8,                    // PDA bump
}
```

### `NftOrigin`

```rust
pub struct NftOrigin {
    pub token_id: u64,               // Unique token ID
    pub original_mint: Pubkey,       // Original mint address
    pub collection: Pubkey,          // Parent collection
    pub chain_of_origin: u64,        // Original chain ID (103=Solana)
    pub metadata_uri: String,        // Current metadata URI
    pub created_at: i64,             // Creation timestamp
    pub bump: u8,                    // PDA bump
}
```

### `Connected`

```rust
pub struct Connected {
    pub collection: Pubkey,          // Parent collection
    pub chain_id: Vec<u8>,          // Connected chain ID
    pub contract_address: Vec<u8>,   // Contract address on chain
    pub bump: u8,                    // PDA bump
}
```

## Events

### `CollectionInitialized`
```rust
pub struct CollectionInitialized {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
}
```

### `TokenMinted`
```rust
pub struct TokenMinted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub name: String,
    pub uri: String,
    pub origin_chain: u64,
    pub is_solana_native: bool,
}
```

### `NftOriginCreated`
```rust
pub struct NftOriginCreated {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub collection: Pubkey,
    pub origin_chain: u64,
    pub metadata_uri: String,
}
```

### `TokenTransferInitiated`
```rust
pub struct TokenTransferInitiated {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub destination_chain: u64,
    pub recipient: Vec<u8>,
    pub sender: Pubkey,
}
```

### `TokenTransferReceived`
```rust
pub struct TokenTransferReceived {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub source_chain: u64,
    pub sender: Vec<u8>,
    pub recipient: Pubkey,
    pub origin_chain: u64,
    pub original_mint: Pubkey,
    pub is_returning: bool,
}
```

## Error Codes

```rust
pub enum UniversalNftError {
    #[msg("Invalid token ID")]
    InvalidTokenId = 6000,
    
    #[msg("Invalid collection authority")]
    InvalidAuthority = 6001,
    
    #[msg("Invalid gateway")]
    InvalidGateway = 6002,
    
    #[msg("Invalid message format")]
    InvalidMessage = 6003,
    
    #[msg("Invalid nonce")]
    InvalidNonce = 6004,
    
    #[msg("Unauthorized gateway")]
    UnauthorizedGateway = 6005,
    
    #[msg("Invalid chain ID")]
    InvalidChainId = 6006,
    
    #[msg("Collection name already exists")]
    CollectionNameExists = 6007,
    
    #[msg("Insufficient funds")]
    InsufficientFunds = 6008,
    
    #[msg("Invalid recipient")]
    InvalidRecipient = 6009,
    
    #[msg("Token not found")]
    TokenNotFound = 6010,
}
```

## Chain IDs

Standard chain identifiers used in the protocol:

- **Solana**: `103` (devnet), `101` (mainnet)
- **Ethereum**: `1` (mainnet), `11155111` (sepolia)
- **BSC**: `56` (mainnet), `97` (testnet)
- **Polygon**: `137` (mainnet), `80001` (mumbai)
- **ZetaChain**: `7000` (mainnet), `7001` (testnet)

## Constants

```rust
pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = // Metaplex program
pub const ZETACHAIN_GATEWAY_PROGRAM_ID: Pubkey = // Gateway program
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_NAME_LENGTH: usize = 64;
pub const MAX_SYMBOL_LENGTH: usize = 10;
```

## Usage Patterns

### Basic NFT Collection Setup

1. Initialize collection with `initialize_collection`
2. Set universal contract with `set_universal`
3. Set connected contracts with `set_connected`
4. Mint NFTs with `mint_nft`

### Cross-Chain Transfer Flow

1. **Outgoing**: Call `transfer_cross_chain` → burns NFT → sends gateway message
2. **Incoming**: Gateway calls `on_call` → mints/recreates NFT → preserves origin

### Origin System Benefits

- **Deterministic Token IDs**: Sequential, collision-free
- **Origin Preservation**: Tracks original mint and chain
- **Metadata Continuity**: Maintains URI across chains
- **Return Detection**: Identifies returning vs new NFTs

## Security Considerations

- All PDAs use proper seeds and bumps
- TSS signature verification for gateway messages
- Nonce-based replay protection
- Authority validation on all operations
- Rent exemption for all accounts
