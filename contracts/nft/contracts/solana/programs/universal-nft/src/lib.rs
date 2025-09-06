use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    keccak,
    secp256k1_recover::secp256k1_recover,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

mod state;
mod instructions;

// Import state module types including NFT Origin
pub use state::{Collection, Connected, NftOrigin, CrossChainMessage, ZetaChainMessage, RevertContext, EVMMessage};
pub use state::{is_supported_chain, validate_chain_id, validate_evm_address, validate_solana_address};

// Import instruction modules
use instructions::*;

// Error definitions
#[error_code]
pub enum UniversalNftError {
    #[msg("Invalid TSS signature")]
    InvalidTssSignature,
    #[msg("Invalid message hash")]
    InvalidMessageHash,
    #[msg("Invalid message")]
    InvalidMessage,
    #[msg("Invalid recipient")]
    InvalidRecipient,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Unauthorized TSS address")]
    UnauthorizedTssAddress,
    #[msg("Invalid nonce")]
    InvalidNonce,
    #[msg("Token does not exist")]
    TokenDoesNotExist,
    #[msg("Not the token owner")]
    NotTokenOwner,
    #[msg("Invalid destination chain")]
    InvalidDestinationChain,
    #[msg("Invalid recipient address")]
    InvalidRecipientAddress,
    #[msg("Insufficient gas amount")]
    InsufficientGasAmount,
    #[msg("Unauthorized gateway call")]
    UnauthorizedGateway,
    #[msg("Unsupported chain")]
    UnsupportedChain,
    #[msg("Invalid token ID")]
    InvalidTokenId,
}

// Metaplex Token Metadata Program ID
pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x0b, 0xa2, 0x31, 0xc2, 0xee, 0x93, 0x5a, 0xc5,
    0xdb, 0x05, 0x01, 0xe4, 0x5e, 0x57, 0x82, 0xd0,
    0x83, 0xfe, 0xbd, 0x48, 0xe9, 0x4b, 0x21, 0x4f,
    0x47, 0x73, 0x0b, 0x11, 0xf1, 0x8b, 0x75, 0xee,
]);

// Universal NFT Program ID - updated with new deployment
declare_id!("6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke");

// ZetaChain Gateway Program ID - deployed on mainnet-beta, testnet, and devnet
// This is the actual deployed ZetaChain gateway program
// Address: ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis
pub const ZETACHAIN_GATEWAY_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x06, 0xa1, 0xe4, 0xdc, 0x88, 0x7a, 0x5b, 0x3f,
    0xbf, 0x0f, 0x5f, 0x1a, 0xc4, 0xb3, 0x6f, 0xf9,
    0x7c, 0x94, 0xf1, 0x3f, 0x42, 0xd8, 0xc4, 0xc5,
    0x7a, 0x71, 0x4c, 0x92, 0x59, 0xbe, 0x63, 0x80
]);

// Gateway PDA account (derived from seeds b"meta" and canonical bump)
pub const GATEWAY_PDA_SEED: &[u8] = b"meta";

#[program]
pub mod universal_nft {
    use super::*;

    /// Initialize a new Universal NFT collection compatible with ZetaChain
    pub fn initialize_collection(
        ctx: Context<InitializeCollection>,
        name: String,
        symbol: String,
        uri: String,
        tss_address: [u8; 20],
    ) -> Result<()> {
        let collection = &mut ctx.accounts.collection;
        let collection_key = collection.key();
        collection.authority = ctx.accounts.authority.key();
        collection.name = name.clone();
        collection.symbol = symbol.clone();
        collection.uri = uri.clone();
        collection.next_token_id = 1;
        collection.tss_address = tss_address;
        collection.gateway_address = None;
        collection.universal_address = None;
        collection.nonce = 0;
        collection.total_minted = 0;
        collection.solana_native_count = 0;
        collection.bump = ctx.bumps.collection;

        // Note: Collection mint and metadata would be created here in production
        // For now, we're focusing on the core NFT functionality

        emit!(CollectionInitialized {
            collection: collection_key,
            authority: ctx.accounts.authority.key(),
            name,
            symbol,
            tss_address,
        });

        Ok(())
    }

    /// Mint a new NFT in the collection with NFT Origin system
    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        // Call the dedicated mint_nft instruction
        instructions::mint_nft::mint_nft(ctx, name, symbol, uri)
    }

    /// Transfer NFT cross-chain with NFT Origin system integration
    pub fn transfer_cross_chain(
        ctx: Context<TransferCrossChain>,
        destination_chain_id: u64,
        recipient: Vec<u8>,
    ) -> Result<()> {
        // Call the dedicated transfer_cross_chain instruction
        instructions::transfer_cross_chain::transfer_cross_chain(ctx, destination_chain_id, recipient)
    }

    /// Handle incoming cross-chain NFT transfer with two-scenario NFT Origin system
    pub fn on_call(
        ctx: Context<OnCall>,
        sender: [u8; 20],
        source_chain_id: u64,
        message: Vec<u8>,
        nonce: u64,
    ) -> Result<()> {
        // Call the dedicated on_call instruction
        instructions::on_call::on_call(ctx, sender, source_chain_id, message, nonce)
    }
    
    /// Receive cross-chain NFT transfer with TSS signature verification
    pub fn receive_cross_chain(
        ctx: Context<ReceiveCrossChain>,
        message_hash: [u8; 32],
        signature: [u8; 64],
        recovery_id: u8,
        message_data: Vec<u8>,
        nonce: u64,
    ) -> Result<()> {
        // Set compute budget for complex operations
        anchor_lang::solana_program::compute_budget::set_compute_unit_limit(400_000)?;

        let collection = &mut ctx.accounts.collection;
        let collection_key = collection.key();
        
        // Enhanced replay protection with comprehensive nonce validation
        require!(
            nonce > collection.nonce,
            UniversalNftError::InvalidNonce
        );
        
        // Validate nonce is not too far in the future (prevent nonce gaps)
        require!(
            nonce <= collection.nonce.saturating_add(1000),
            UniversalNftError::InvalidNonce
        );
        
        // Verify message hash integrity
        let computed_hash = keccak::hash(&message_data);
        require!(
            computed_hash.to_bytes() == message_hash,
            UniversalNftError::InvalidMessageHash
        );
        
        // Enhanced TSS signature verification with proper error handling
        let recovered_pubkey = secp256k1_recover(&message_hash, recovery_id, &signature)
            .map_err(|_| UniversalNftError::InvalidTssSignature)?;
        
        // Validate recovery ID is in valid range (0-3)
        require!(
            recovery_id <= 3,
            UniversalNftError::InvalidTssSignature
        );
        
        let recovered_address = pubkey_to_eth_address(&recovered_pubkey.0)?;
        
        require!(
            recovered_address == collection.tss_address,
            UniversalNftError::UnauthorizedTssAddress
        );

        // Enhanced cross-chain message decoding with fallback mechanisms
        let cross_chain_message = decode_cross_chain_message(&message_data)?;

        // Comprehensive recipient validation
        let expected_recipient = ctx.accounts.recipient.key();
        validate_recipient_address(&cross_chain_message.recipient, &expected_recipient)?;
        
        // Validate token ID format and constraints
        require!(
            cross_chain_message.token_id > 0,
            UniversalNftError::InvalidTokenId
        );
        
        // Validate URI format and length
        require!(
            !cross_chain_message.uri.is_empty() && cross_chain_message.uri.len() <= 200,
            UniversalNftError::InvalidMessage
        );
        
        // Update nonce to prevent replay attacks
        collection.nonce = nonce;

        // Extract values before mutable borrow
        let collection_authority = collection.authority;
        let collection_name = collection.name.clone();
        let collection_bump = collection.bump;

        // Mint the NFT to the recipient with proper error handling
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.collection.to_account_info(),
            },
        );

        let seeds = &[
            b"collection",
            collection_authority.as_ref(),
            collection_name.as_bytes(),
            &[collection_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        mint_to(cpi_ctx.with_signer(signer_seeds), 1)
            .map_err(|_| UniversalNftError::TokenDoesNotExist)?;

        // Increment collection statistics
        collection.increment_total_minted()?;

        // Determine if this is a returning NFT or new arrival
        let (origin_chain, original_mint, is_returning) = determine_nft_origin(
            cross_chain_message.token_id,
            &cross_chain_message.sender,
        )?;

        emit!(TokenTransferReceived {
            collection: collection_key,
            token_id: cross_chain_message.token_id,
            recipient: ctx.accounts.recipient.key(),
            uri: cross_chain_message.uri,
            original_sender: cross_chain_message.sender,
            nonce,
            origin_chain,
            original_mint,
            is_returning,
        });

        Ok(())
    }

    /// Set the universal contract address on ZetaChain
    pub fn set_universal(
        ctx: Context<SetUniversalContext>,
        universal_address: Pubkey,
    ) -> Result<()> {
        // Call the dedicated set_universal instruction
        instructions::set_universal::set_universal(ctx, universal_address)
    }

    /// Set connected contract address for a specific chain
    pub fn set_connected(
        ctx: Context<SetConnectedContext>,
        chain_id: Vec<u8>,
        contract_address: Vec<u8>,
    ) -> Result<()> {
        // Call the dedicated set_connected instruction
        instructions::set_connected::set_connected(ctx, chain_id, contract_address)
    }

    /// Handle failed cross-chain transfers by minting NFT back to original sender
    pub fn on_revert(
        ctx: Context<OnRevertContext>,
        token_id: u64,
        uri: String,
        original_sender: Pubkey,
        refund_amount: u64,
    ) -> Result<()> {
        // Call the dedicated on_revert instruction
        instructions::on_revert::on_revert(ctx, token_id, uri, original_sender, refund_amount)
    }
}

/// NFT Origin helper functions
/// Find NFT Origin PDA for a given token ID
pub fn find_nft_origin_pda(program_id: &Pubkey, token_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"nft_origin",
            &token_id.to_le_bytes(),
        ],
        program_id,
    )
}

/// Generate deterministic token ID using mint + block + next_token_id as described in documentation
pub fn generate_deterministic_token_id(mint: &Pubkey, block_number: u64, next_token_id: u64) -> u64 {
    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(&mint.to_bytes());
    hash_input.extend_from_slice(&block_number.to_le_bytes());
    hash_input.extend_from_slice(&next_token_id.to_le_bytes());
    
    let hash = keccak::hash(&hash_input);
    u64::from_le_bytes([
        hash.0[0], hash.0[1], hash.0[2], hash.0[3],
        hash.0[4], hash.0[5], hash.0[6], hash.0[7]
    ])
}

/// Check if an NFT Origin PDA exists for a given token ID
pub fn nft_origin_exists(_program_id: &Pubkey, token_id: u64) -> bool {
    let (_origin_pda, _) = find_nft_origin_pda(&crate::ID, token_id);
    // In a real implementation, this would check if the account exists on-chain
    // For now, this is a placeholder that would be implemented with proper account checks
    false
}

/// Get current Solana chain ID based on cluster
pub fn get_current_chain_id() -> u64 {
    // This would be determined by the cluster configuration
    // For now, defaulting to devnet
    103 // Solana devnet
}

// Enhanced helper function to convert secp256k1 public key to Ethereum address
fn pubkey_to_eth_address(pubkey: &[u8; 64]) -> Result<[u8; 20]> {
    use anchor_lang::solana_program::keccak::hash;
    
    // Validate public key format
    require!(
        pubkey.len() == 64,
        UniversalNftError::InvalidTssSignature
    );
    
    // Validate public key is not all zeros
    require!(
        !pubkey.iter().all(|&b| b == 0),
        UniversalNftError::InvalidTssSignature
    );
    
    // Hash the uncompressed public key (64 bytes)
    let hash_result = hash(pubkey);
    let hash_bytes = hash_result.to_bytes();
    
    // Take the last 20 bytes as Ethereum address
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash_bytes[12..32]);
    
    // Validate address is not zero address
    require!(
        !address.iter().all(|&b| b == 0),
        UniversalNftError::InvalidTssSignature
    );
    
    Ok(address)
}

#[derive(Accounts)]
#[instruction(name: String, _symbol: String, _uri: String, tss_address: [u8; 20])]
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

    /// CHECK: Collection mint - not used in simplified version
    pub collection_mint: UncheckedAccount<'info>,

    /// CHECK: Collection token account - not used in simplified version
    pub collection_token_account: UncheckedAccount<'info>,

    /// CHECK: Metadata account - not used in simplified version
    pub collection_metadata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}




#[derive(Accounts)]
pub struct ReceiveCrossChain<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: NFT recipient account
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Metadata account for the NFT
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum GatewayInstruction {
    Call {
        receiver: [u8; 20],
        amount: u64,
        message: Vec<u8>,
    },
}

#[event]
pub struct CollectionInitialized {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub tss_address: [u8; 20],
}

#[event]
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

#[event]
pub struct TokenTransfer {
    pub collection: Pubkey,
    pub token_id: u64,
    pub destination_chain_id: u64,
    pub recipient: Vec<u8>,
    pub uri: String,
    pub sender: Pubkey,
    pub message: Vec<u8>,
    pub origin_chain: Option<u64>,
    pub original_mint: Option<Pubkey>,
    pub is_returning: bool,
}

#[event]
pub struct TokenTransferReceived {
    pub collection: Pubkey,
    pub token_id: u64,
    pub recipient: Pubkey,
    pub uri: String,
    pub original_sender: Vec<u8>,
    pub nonce: u64,
    pub origin_chain: Option<u64>,
    pub original_mint: Option<Pubkey>,
    pub is_returning: bool,
}

#[event]
pub struct TokenTransferReverted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub sender: Pubkey,
    pub uri: String,
    pub refund_amount: u64,
    pub origin_chain: Option<u64>,
    pub original_mint: Option<Pubkey>,
}

/// NFT Origin specific events
#[event]
pub struct NftOriginCreated {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub collection: Pubkey,
    pub origin_chain: u64,
    pub metadata_uri: String,
}

#[event]
pub struct NftOriginUpdated {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub updated_fields: Vec<String>,
}

#[event]
pub struct NftReturningToSolana {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub new_mint: Pubkey,
    pub metadata_preserved: bool,
}

#[event]
pub struct CrossChainCycleCompleted {
    pub token_id: u64,
    pub origin_chain: u64,
    pub destination_chain: u64,
    pub cycle_count: u64,
}

#[event]
pub struct SetUniversal {
    pub collection: Pubkey,
    pub universal_address: Pubkey,
}

#[event]
pub struct SetConnected {
    pub collection: Pubkey,
    pub chain_id: Vec<u8>,
    pub contract_address: Vec<u8>,
}


/// Enhanced cross-chain message decoder with multiple format support
fn decode_cross_chain_message(message: &[u8]) -> Result<CrossChainMessage> {
    // Validate minimum message length
    require!(
        message.len() >= 32,
        UniversalNftError::InvalidMessage
    );
    
    // Try ZetaChain message format first (most common)
    if let Ok(zetachain_msg) = try_decode_zetachain_message(message) {
        return Ok(convert_zetachain_to_cross_chain(zetachain_msg)?);
    }
    
    // Try ABI-encoded format (for EVM chains)
    if let Ok(abi_msg) = try_decode_abi_message(message) {
        return Ok(abi_msg);
    }
    
    // Try Borsh-encoded format (for Solana and other chains)
    if let Ok(borsh_msg) = try_decode_borsh_message(message) {
        return Ok(borsh_msg);
    }
    
    // Try legacy format for backward compatibility
    try_decode_legacy_message(message)
}

/// Try to decode message as ZetaChain format
fn try_decode_zetachain_message(message: &[u8]) -> Result<ZetaChainMessage> {
    ZetaChainMessage::try_from_slice(message)
        .map_err(|_| UniversalNftError::InvalidMessageHash)
}

/// Convert ZetaChain message to CrossChainMessage format
fn convert_zetachain_to_cross_chain(zetachain_msg: ZetaChainMessage) -> Result<CrossChainMessage> {
    // Validate destination address format
    let recipient = if zetachain_msg.destination_address.len() == 20 {
        // EVM address - pad to 32 bytes for Solana compatibility
        let mut padded = vec![0u8; 12];
        padded.extend_from_slice(&zetachain_msg.destination_address);
        padded
    } else {
        zetachain_msg.destination_address.to_vec()
    };
    
    // Validate sender format
    let sender = if zetachain_msg.sender.len() == 32 {
        // Solana address - convert to EVM format for consistency
        zetachain_msg.sender[..20].to_vec()
    } else {
        zetachain_msg.sender.to_vec()
    };
    
    Ok(CrossChainMessage {
        token_id: zetachain_msg.token_id,
        uri: zetachain_msg.uri,
        recipient,
        destination_chain: zetachain_msg.destination_chain_id.to_le_bytes().to_vec(),
        sender,
    })
}

/// Enhanced ABI message decoder with proper validation
fn try_decode_abi_message(message: &[u8]) -> Result<CrossChainMessage> {
    // Enhanced ABI decoder for EVM chain messages
    // Expected format: [token_id(32), uri_offset(32), recipient(32), sender(20), uri_len(32), uri(variable)]
    require!(message.len() >= 148, UniversalNftError::InvalidMessageHash);
    
    let mut offset = 0;
    
    // Extract token_id (32 bytes, big-endian for ABI compatibility)
    let token_id_bytes: [u8; 32] = message[offset..offset + 32]
        .try_into()
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    let token_id = u64::from_be_bytes([
        token_id_bytes[24], token_id_bytes[25], token_id_bytes[26], token_id_bytes[27],
        token_id_bytes[28], token_id_bytes[29], token_id_bytes[30], token_id_bytes[31]
    ]);
    offset += 32;
    
    // Skip URI offset (32 bytes)
    offset += 32;
    
    // Extract recipient (32 bytes)
    let recipient_bytes: [u8; 32] = message[offset..offset + 32]
        .try_into()
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    offset += 32;
    
    // Extract sender (20 bytes EVM address)
    let sender_bytes: [u8; 20] = message[offset..offset + 20]
        .try_into()
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    offset += 20;
    
    // Skip padding to align to 32 bytes
    offset += 12;
    
    // Extract URI length (32 bytes)
    let uri_len_bytes: [u8; 32] = message[offset..offset + 32]
        .try_into()
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    let uri_len = u32::from_be_bytes([
        uri_len_bytes[28], uri_len_bytes[29], uri_len_bytes[30], uri_len_bytes[31]
    ]) as usize;
    offset += 32;
    
    // Validate URI length
    require!(
        uri_len <= 200 && message.len() >= offset + uri_len,
        UniversalNftError::InvalidMessageHash
    );
    
    // Extract URI
    let uri = String::from_utf8(message[offset..offset + uri_len].to_vec())
        .map_err(|_| UniversalNftError::InvalidMessage)?;
    
    // Validate URI is not empty
    require!(!uri.is_empty(), UniversalNftError::InvalidMessage);
    
    Ok(CrossChainMessage {
        token_id,
        uri,
        recipient: recipient_bytes.to_vec(),
        destination_chain: get_current_chain_id().to_le_bytes().to_vec(),
        sender: sender_bytes.to_vec(),
    })
}

/// Enhanced Borsh message decoder with validation
fn try_decode_borsh_message(message: &[u8]) -> Result<CrossChainMessage> {
    let cross_chain_message = CrossChainMessage::try_from_slice(message)
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    
    // Validate token ID
    require!(
        cross_chain_message.token_id > 0,
        UniversalNftError::InvalidTokenId
    );
    
    // Validate URI
    require!(
        !cross_chain_message.uri.is_empty() && cross_chain_message.uri.len() <= 200,
        UniversalNftError::InvalidMessage
    );
    
    // Validate recipient address format
    require!(
        cross_chain_message.recipient.len() == 32 || cross_chain_message.recipient.len() == 20,
        UniversalNftError::InvalidRecipientAddress
    );
    
    // Validate sender address format
    require!(
        cross_chain_message.sender.len() == 32 || cross_chain_message.sender.len() == 20,
        UniversalNftError::InvalidRecipientAddress
    );
    
    Ok(cross_chain_message)
}

/// Legacy message format decoder for backward compatibility
fn try_decode_legacy_message(message: &[u8]) -> Result<CrossChainMessage> {
    // Legacy format: [token_id(8), uri_len(4), uri(variable), recipient(32), sender(20)]
    require!(message.len() >= 64, UniversalNftError::InvalidMessageHash);
    
    let mut offset = 0;
    
    // Extract token_id (8 bytes, little-endian)
    let token_id = u64::from_le_bytes(
        message[offset..offset + 8]
            .try_into()
            .map_err(|_| UniversalNftError::InvalidMessageHash)?
    );
    offset += 8;
    
    // Extract URI length and URI
    let uri_len = u32::from_le_bytes(
        message[offset..offset + 4]
            .try_into()
            .map_err(|_| UniversalNftError::InvalidMessageHash)?
    ) as usize;
    offset += 4;
    
    require!(
        uri_len <= 200 && message.len() >= offset + uri_len + 52,
        UniversalNftError::InvalidMessageHash
    );
    
    let uri = String::from_utf8(message[offset..offset + uri_len].to_vec())
        .map_err(|_| UniversalNftError::InvalidMessage)?;
    offset += uri_len;
    
    // Extract recipient (32 bytes)
    let recipient_bytes: [u8; 32] = message[offset..offset + 32]
        .try_into()
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    offset += 32;
    
    // Extract sender (20 bytes)
    let sender_bytes: [u8; 20] = message[offset..offset + 20]
        .try_into()
        .map_err(|_| UniversalNftError::InvalidMessageHash)?;
    
    Ok(CrossChainMessage {
        token_id,
        uri,
        recipient: recipient_bytes.to_vec(),
        destination_chain: get_current_chain_id().to_le_bytes().to_vec(),
        sender: sender_bytes.to_vec(),
    })
}

/// Validate recipient address format and compatibility
fn validate_recipient_address(message_recipient: &[u8], expected_recipient: &Pubkey) -> Result<()> {
    if message_recipient.len() == 32 {
        // Solana address format
        let recipient_pubkey = Pubkey::new_from_array(
            message_recipient.try_into()
                .map_err(|_| UniversalNftError::InvalidRecipientAddress)?
        );
        require!(
            recipient_pubkey == *expected_recipient,
            UniversalNftError::InvalidRecipient
        );
    } else if message_recipient.len() == 20 {
        // EVM address format - derive corresponding Solana address
        // This is a simplified approach - real implementation would use proper derivation
        let derived_pubkey = derive_solana_address_from_evm(message_recipient)?;
        require!(
            derived_pubkey == *expected_recipient,
            UniversalNftError::InvalidRecipient
        );
    } else {
        return Err(UniversalNftError::InvalidRecipientAddress.into());
    }
    
    Ok(())
}

/// Derive Solana address from EVM address (simplified approach)
fn derive_solana_address_from_evm(evm_address: &[u8]) -> Result<Pubkey> {
    require!(evm_address.len() == 20, UniversalNftError::InvalidRecipientAddress);
    
    // Create a deterministic Solana address from EVM address
    let mut seed_data = Vec::new();
    seed_data.extend_from_slice(b"evm_derived");
    seed_data.extend_from_slice(evm_address);
    
    let hash = keccak::hash(&seed_data);
    Ok(Pubkey::new_from_array(hash.to_bytes()))
}

/// Determine NFT origin information for tracking
fn determine_nft_origin(token_id: u64, sender: &[u8]) -> Result<(Option<u64>, Option<Pubkey>, bool)> {
    // Check if NFT Origin PDA exists for this token ID
    let (origin_pda, _) = find_nft_origin_pda(&crate::ID, token_id);
    
    // In a real implementation, this would check if the account exists on-chain
    // For now, we'll determine based on sender format and token ID patterns
    
    // If sender is Solana format (32 bytes), likely returning to origin
    if sender.len() == 32 {
        let sender_pubkey = Pubkey::new_from_array(
            sender.try_into()
                .map_err(|_| UniversalNftError::InvalidRecipientAddress)?
        );
        return Ok((Some(get_current_chain_id()), Some(sender_pubkey), true));
    }
    
    // If sender is EVM format (20 bytes), likely new arrival
    if sender.len() == 20 {
        // Determine origin chain based on token ID patterns or other metadata
        let origin_chain = determine_origin_chain_from_token_id(token_id);
        return Ok((Some(origin_chain), None, false));
    }
    
    // Default case
    Ok((None, None, false))
}

/// Determine origin chain from token ID patterns
fn determine_origin_chain_from_token_id(token_id: u64) -> u64 {
    // This is a simplified approach - real implementation would use proper origin tracking
    // Token ID ranges could indicate different origin chains
    match token_id {
        1..=1000000 => state::CHAIN_ID_ETHEREUM,
        1000001..=2000000 => state::CHAIN_ID_BSC,
        2000001..=3000000 => state::CHAIN_ID_POLYGON,
        3000001..=4000000 => state::CHAIN_ID_BASE,
        4000001..=5000000 => state::CHAIN_ID_ARBITRUM,
        5000001..=6000000 => state::CHAIN_ID_OPTIMISM,
        _ => state::CHAIN_ID_ZETACHAIN,
    }
}

/// Calculate gas fee based on destination chain and gas amount
pub fn calculate_gas_fee(destination_chain: u64, gas_amount: u64) -> Result<u64> {
    // Enhanced gas calculation with dynamic pricing
    let base_gas: u64 = match destination_chain {
        84532 => 100_000,    // Base Sepolia - higher gas for L2
        11155111 => 150_000, // Ethereum Sepolia - highest gas
        7001 => 50_000,      // ZetaChain testnet - lower gas
        97 => 80_000,        // BSC testnet
        80001 => 80_000,     // Polygon Mumbai
        421614 => 100_000,   // Arbitrum Sepolia
        11155420 => 100_000, // Optimism Sepolia
        _ => 100_000,        // Default gas
    };
    
    // Validate gas amount is reasonable
    require!(
        gas_amount > 0 && gas_amount <= 1_000_000,
        UniversalNftError::InsufficientGasAmount
    );
    
    let total_fee = base_gas
        .checked_mul(gas_amount)
        .ok_or(UniversalNftError::InsufficientGasAmount)?;
    
    // Apply dynamic pricing based on network congestion (simplified)
    let congestion_multiplier = get_congestion_multiplier(destination_chain);
    let adjusted_fee = total_fee
        .checked_mul(congestion_multiplier)
        .and_then(|f| f.checked_div(100))
        .ok_or(UniversalNftError::InsufficientGasAmount)?;
    
    // Ensure minimum and maximum gas fees
    let min_fee = 10_000_000; // 0.01 SOL minimum
    let max_fee = 1_000_000_000; // 1 SOL maximum
    Ok(adjusted_fee.max(min_fee).min(max_fee))
}

/// Get congestion multiplier for dynamic gas pricing
fn get_congestion_multiplier(destination_chain: u64) -> u64 {
    // Simplified congestion multiplier (percentage)
    // Real implementation would query network conditions
    match destination_chain {
        11155111 => 150, // Ethereum often congested
        84532 => 120,    // Base moderate congestion
        7001 => 100,     // ZetaChain typically low congestion
        _ => 110,        // Default moderate congestion
    }
}
