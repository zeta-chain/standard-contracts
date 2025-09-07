use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    clock::Clock,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    sysvar,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, Burn, Mint, Token, TokenAccount},
};
use solana_program::program_option::COption;

use crate::state::{Collection, NftOrigin, Connected, convert_address_format};
use crate::utils::{is_supported_chain, serialize_gateway_call_data};
use crate::{
    UniversalNftError, 
    ZETACHAIN_GATEWAY_PROGRAM_ID, 
    TOKEN_METADATA_PROGRAM_ID,
    calculate_gas_fee,
    get_current_chain_id,
    GATEWAY_PDA_SEED,
};
use crate::TokenTransfer;

/// Transfer NFT cross-chain with NFT Origin system integration
pub fn transfer_cross_chain(
    ctx: Context<TransferCrossChain>,
    destination_chain_id: u64,
    recipient: Vec<u8>,
) -> Result<()> {
    let collection = &mut ctx.accounts.collection;
    let nft_origin = &ctx.accounts.nft_origin;
    let sender = ctx.accounts.sender.key();
    let collection_key = collection.key();

    // Validate recipient address format for destination chain
    let formatted_recipient = convert_address_format(&recipient, destination_chain_id)?;

    // Validate NFT ownership through token account
    require!(
        ctx.accounts.nft_token_account.amount == 1,
        UniversalNftError::TokenDoesNotExist
    );
    require!(
        ctx.accounts.nft_token_account.owner == sender,
        UniversalNftError::NotTokenOwner
    );

    // Validate NFT exists in origin system (token_id 0 is valid for sequential IDs)
    // Remove over-restrictive check since token_id can be 0 for the first NFT

    // Validate the NFT mint matches the origin system
    require!(
        nft_origin.original_mint == ctx.accounts.nft_mint.key() || 
        nft_origin.collection == collection_key,
        UniversalNftError::InvalidTokenId
    );

    // Validate gateway PDA is properly derived
    let (expected_gateway_pda, _) = Pubkey::find_program_address(
        &[GATEWAY_PDA_SEED],
        &ZETACHAIN_GATEWAY_PROGRAM_ID,
    );
    require!(
        ctx.accounts.gateway_pda.key() == expected_gateway_pda,
        UniversalNftError::UnauthorizedGateway
    );

    // Extract origin information from the NFT Origin PDA
    let token_id = nft_origin.token_id;
    let origin_chain = nft_origin.chain_of_origin;
    let original_mint = nft_origin.original_mint;
    let metadata_uri = nft_origin.metadata_uri.clone();
    let is_solana_native = nft_origin.is_solana_native();
    let is_returning = destination_chain_id == origin_chain;

    // Calculate gas fee for cross-chain transfer using canonical function
    let message_size = metadata_uri.len() + 200; // Approximate message size with overhead
    let gas_fee = calculate_gas_fee(destination_chain_id, message_size as u64)?;

    // Validate transfer parameters with actual gas fee
    validate_transfer_parameters(
        destination_chain_id,
        &formatted_recipient,
        gas_fee,
        ctx.accounts.sender.lamports(),
    )?;

    // Validate connected contract exists for destination chain
    require!(
        ctx.accounts.connected.contract_address.len() == 20,
        UniversalNftError::InvalidDestinationChain
    );

    // Create enhanced cross-chain message with origin information
    let cross_chain_message = create_cross_chain_message_with_origin(
        destination_chain_id,
        &formatted_recipient,
        token_id,
        &metadata_uri,
        &sender.to_bytes(),
        origin_chain,
        original_mint,
        is_solana_native,
    )?;

    // Burn the NFT token
    let _collection_authority = collection.authority;
    let _collection_name = collection.name.clone();
    let _collection_bump = collection.bump;

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.nft_mint.to_account_info(),
            from: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        },
    );
    burn(cpi_ctx, 1)?;

    // Update collection statistics when burning
    // Note: We don't decrement total_minted as it represents historical count
    // Only decrement solana_native_count if this was a Solana-native NFT leaving
    if is_solana_native && destination_chain_id != get_current_chain_id() {
        // NFT is leaving Solana for another chain
        collection.solana_native_count = collection.solana_native_count.saturating_sub(1);
    }

    // Prepare enhanced gateway message for cross-chain transfer
    let gateway_message = prepare_gateway_message(destination_chain_id, &cross_chain_message)?;

    // Validate message size doesn't exceed limits
    require!(
        gateway_message.len() <= 10240, // 10KB limit
        UniversalNftError::InvalidMessage
    );

    // Call ZetaChain gateway for cross-chain transfer using proper deposit_and_call
    let destination_address = derive_destination_contract_address(&ctx.accounts.connected)?;
    let gateway_instruction = create_gateway_instruction(
        &ZETACHAIN_GATEWAY_PROGRAM_ID,
        &ctx.accounts.gateway_pda.key(),
        &ctx.accounts.sender.key(),
        destination_chain_id,
        destination_address.to_vec(),
        gateway_message,
        gas_fee,
    )?;

    // Execute gateway call with proper account structure - must match instruction account metas
    invoke_signed(
        &gateway_instruction,
        &[
            ctx.accounts.sender.to_account_info(),
            ctx.accounts.gateway_pda.to_account_info(),
            ctx.accounts.gateway.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[],
    )?;

    // Note: Gas fee is automatically transferred by the gateway instruction

    // Emit transfer event with origin information
    let clock = Clock::get().map_err(|_| UniversalNftError::InvalidMessage)?;
    emit!(TokenTransfer {
        collection: collection_key,
        token_id,
        destination_chain_id,
        recipient: formatted_recipient,
        uri: metadata_uri,
        sender,
        message: cross_chain_message,
        timestamp: clock.unix_timestamp,
        origin_chain,
        origin_mint: original_mint,
        is_returning,
    });

    msg!(
        "NFT transferred cross-chain: token_id={}, destination_chain={}, origin_chain={}, is_returning={}, gas_fee={}",
        token_id,
        destination_chain_id,
        origin_chain,
        is_returning,
        gas_fee
    );

    Ok(())
}

/// Validate cross-chain transfer parameters
fn validate_transfer_parameters(
    destination_chain_id: u64,
    recipient: &[u8],
    gas_fee: u64,
    sender_balance: u64,
) -> Result<()> {
    // Validate destination chain
    require!(
        is_supported_chain(destination_chain_id),
        UniversalNftError::UnsupportedChain
    );

    // Validate recipient address format
    match destination_chain_id {
        // EVM chains require 20-byte addresses
        1 | 56 | 137 | 8453 | 42161 | 10 | 11155111 | 97 | 80001 | 84532 | 421614 | 11155420 => {
            require!(
                recipient.len() == 20,
                UniversalNftError::InvalidRecipientAddress
            );
        },
        // ZetaChain can accept both 20-byte and 32-byte addresses
        7000 | 7001 => {
            require!(
                recipient.len() == 20 || recipient.len() == 32,
                UniversalNftError::InvalidRecipientAddress
            );
        },
        _ => {
            require!(
                recipient.len() >= 20 && recipient.len() <= 32,
                UniversalNftError::InvalidRecipientAddress
            );
        }
    }

    // Validate gas fee is reasonable
    require!(
        gas_fee >= 1_000_000 && gas_fee <= 1_000_000_000, // Between 0.001 and 1 SOL
        UniversalNftError::InsufficientGasAmount
    );

    // Validate sender has sufficient balance
    require!(
        sender_balance >= gas_fee + 5_000_000, // Gas fee + 0.005 SOL buffer
        UniversalNftError::InsufficientGasAmount
    );

    Ok(())
}

/// Create enhanced cross-chain message with origin information
fn create_cross_chain_message_with_origin(
    destination_chain_id: u64,
    recipient: &[u8],
    token_id: u64,
    metadata_uri: &str,
    sender: &[u8],
    origin_chain: u64,
    original_mint: Pubkey,
    is_solana_native: bool,
) -> Result<Vec<u8>> {
    // Create message based on destination chain type
    match destination_chain_id {
        // EVM chains - use EVM message format with origin data
        1 | 56 | 137 | 8453 | 42161 | 10 | // Mainnets
        11155111 | 97 | 80001 | 84532 | 421614 | 11155420 => { // Testnets
            let evm_recipient: [u8; 20] = recipient[..20].try_into()
                .map_err(|_| UniversalNftError::InvalidRecipientAddress)?;
            let evm_sender: [u8; 20] = sender[..20].try_into()
                .map_err(|_| UniversalNftError::InvalidRecipientAddress)?;

            // Enhanced EVM message with origin information
            let enhanced_message = EnhancedEVMMessage {
                token_id,
                recipient: evm_recipient,
                uri: metadata_uri.to_string(),
                sender: evm_sender,
                origin_chain,
                original_mint: original_mint.to_bytes(),
                is_solana_native,
            };

            enhanced_message.try_to_vec()
                .map_err(|_| UniversalNftError::InvalidMessage.into())
        },
        // ZetaChain - use ZetaChain message format
        7000 | 7001 => {
            let zetachain_recipient: [u8; 20] = if recipient.len() == 20 {
                recipient.try_into().unwrap()
            } else {
                recipient[12..32].try_into()
                    .map_err(|_| UniversalNftError::InvalidRecipientAddress)?
            };

            let zetachain_sender: [u8; 32] = sender.try_into()
                .map_err(|_| UniversalNftError::InvalidRecipientAddress)?;

            // Enhanced ZetaChain message with origin information
            let enhanced_message = EnhancedZetaChainMessage {
                destination_chain_id,
                destination_address: zetachain_recipient,
                destination_gas_limit: 100_000,
                message: metadata_uri.as_bytes().to_vec(),
                token_id,
                uri: metadata_uri.to_string(),
                sender: zetachain_sender,
                origin_chain,
                original_mint: original_mint.to_bytes(),
                is_solana_native,
            };

            enhanced_message.try_to_vec()
                .map_err(|_| UniversalNftError::InvalidMessage.into())
        },
        _ => {
            // Generic cross-chain message for other chains
            let enhanced_message = EnhancedCrossChainMessage {
                destination_chain: destination_chain_id.to_le_bytes().to_vec(),
                recipient: recipient.to_vec(),
                token_id,
                uri: metadata_uri.to_string(),
                sender: sender.to_vec(),
                origin_chain,
                original_mint: original_mint.to_bytes(),
                is_solana_native,
            };

            enhanced_message.try_to_vec()
                .map_err(|_| UniversalNftError::InvalidMessage.into())
        }
    }
}

/// Prepare gateway message for ZetaChain with proper formatting
fn prepare_gateway_message(destination_chain_id: u64, message: &[u8]) -> Result<Vec<u8>> {
    // Create a structured message for ZetaChain gateway
    let mut gateway_message = Vec::new();
    
    // Add message type identifier for NFT transfer
    gateway_message.extend_from_slice(b"NFT_TRANSFER");
    
    // Add destination chain ID (8 bytes)
    gateway_message.extend_from_slice(&destination_chain_id.to_le_bytes());
    
    // Add message length (4 bytes)
    gateway_message.extend_from_slice(&(message.len() as u32).to_le_bytes());
    
    // Add the actual message data
    gateway_message.extend_from_slice(message);
    
    // Add checksum for integrity verification
    let checksum = anchor_lang::solana_program::keccak::hash(&gateway_message);
    gateway_message.extend_from_slice(&checksum.to_bytes()[..4]);
    Ok(gateway_message)
}

/// Create proper ZetaChain gateway instruction using canonical helpers
fn create_gateway_instruction(
    gateway_program_id: &Pubkey,
    gateway_pda: &Pubkey,
    sender: &Pubkey,
    destination_chain_id: u64,
    destination_address: Vec<u8>,
    message: Vec<u8>,
    gas_fee: u64,
) -> Result<Instruction> {
    // Use canonical gateway instruction builder if available
    // For now, using simplified instruction format that matches gateway expectations
    let instruction_data = serialize_gateway_call_data(
        destination_chain_id,
        &destination_address,
        &message,
        gas_fee,
    )?;

    Ok(Instruction {
        program_id: ZETACHAIN_GATEWAY_PROGRAM_ID,
        accounts: vec![
            // Sender account (signer, writable - source of SOL)
            AccountMeta::new(*sender, true),
            // Gateway PDA (writable - receives SOL)
            AccountMeta::new(*gateway_pda, false),
            // Gateway program account
            AccountMeta::new_readonly(ZETACHAIN_GATEWAY_PROGRAM_ID, false),
            // System program
            AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
            // Rent sysvar
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: instruction_data,
    })
}

/// Derive the destination contract address for the given chain
fn derive_destination_contract_address(connected: &Connected) -> Result<[u8; 20]> {
    // Use Connected PDA to get the actual contract address for the destination chain
    let contract_address = connected.contract_address.clone();
    
    // Convert Vec<u8> to [u8; 20] for EVM address
    let mut address_array = [0u8; 20];
    if contract_address.len() >= 20 {
        address_array.copy_from_slice(&contract_address[..20]);
    } else {
        address_array[..contract_address.len()].copy_from_slice(&contract_address);
    }
    Ok(address_array)
}

/// Create the instruction data for deposit_and_call
fn create_deposit_and_call_data(
    amount: u64,
    receiver: [u8; 20],
    message: Vec<u8>,
    revert_options: Option<RevertOptions>,
) -> Result<Vec<u8>> {
    // Calculate the instruction discriminator for "global:deposit_and_call"
    let discriminator = anchor_lang::solana_program::keccak::hash(b"global:deposit_and_call");
    let discriminator_bytes = &discriminator.to_bytes()[..8];
    
    // Serialize the instruction data
    let deposit_and_call_data = DepositAndCallData {
        amount,
        receiver,
        message,
        revert_options,
    };
    
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(discriminator_bytes);
    instruction_data.extend_from_slice(&deposit_and_call_data.try_to_vec()
        .map_err(|_| UniversalNftError::InvalidMessage)?);
    
    Ok(instruction_data)
}


/// Verify TSS signature for enhanced security
pub fn verify_tss_signature(
    message_hash: &[u8; 32],
    signature: &[u8; 64],
    recovery_id: u8,
    expected_tss_address: &[u8; 20],
) -> Result<bool> {
    
    // Recover the public key from the signature using Solana syscall
    let recovered_pubkey = anchor_lang::solana_program::secp256k1_recover::secp256k1_recover(message_hash, recovery_id, signature)
        .map_err(|_| UniversalNftError::InvalidTssSignature)?;
    
    // Convert recovered public key to Ethereum address
    let recovered_address = pubkey_to_eth_address(&recovered_pubkey.0)?;
    
    // Verify the recovered address matches the expected TSS address
    Ok(recovered_address == *expected_tss_address)
}

/// Convert secp256k1 public key to Ethereum address
fn pubkey_to_eth_address(pubkey: &[u8; 64]) -> Result<[u8; 20]> {
    let hash_result = anchor_lang::solana_program::keccak::hash(pubkey);
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash_result.to_bytes()[12..]);
    Ok(address)
}

/// Data structures for ZetaChain gateway integration

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositAndCallData {
    pub amount: u64,
    pub receiver: [u8; 20],
    pub message: Vec<u8>,
    pub revert_options: Option<RevertOptions>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RevertOptions {
    pub revert_address: [u8; 20],
    pub call_on_revert: bool,
    pub abort_address: [u8; 20],
    pub revert_message: Vec<u8>,
    pub on_revert_gas_limit: u64,
}


/// Enhanced message structures with origin information

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EnhancedEVMMessage {
    pub token_id: u64,
    pub recipient: [u8; 20],
    pub uri: String,
    pub sender: [u8; 20],
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EnhancedZetaChainMessage {
    pub destination_chain_id: u64,
    pub destination_address: [u8; 20],
    pub destination_gas_limit: u64,
    pub message: Vec<u8>,
    pub token_id: u64,
    pub uri: String,
    pub sender: [u8; 32],
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EnhancedCrossChainMessage {
    pub destination_chain: Vec<u8>,
    pub recipient: Vec<u8>,
    pub token_id: u64,
    pub uri: String,
    pub sender: Vec<u8>,
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
}

/// Account structure for TransferCrossChain instruction
#[derive(Accounts)]
#[instruction(destination_chain_id: u64, recipient: Vec<u8>)]
pub struct TransferCrossChain<'info> {
    /// Collection account
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    /// NFT Origin PDA account - must exist for the NFT being transferred
    #[account(
        seeds = [b"nft_origin", nft_origin.token_id.to_le_bytes().as_ref()],
        bump = nft_origin.bump,
        constraint = nft_origin.collection == collection.key() @ UniversalNftError::InvalidTokenId
    )]
    pub nft_origin: Account<'info, NftOrigin>,

    /// Connected contract account for destination chain validation
    #[account(
        seeds = [b"connected", collection.key().as_ref(), destination_chain_id.to_le_bytes().as_slice()],
        bump = connected.bump
    )]
    pub connected: Account<'info, Connected>,

    /// NFT mint account
    #[account(
        mut,
        constraint = nft_mint.supply == 1 @ UniversalNftError::TokenDoesNotExist,
        constraint = matches!(nft_mint.mint_authority, COption::Some(auth) if auth == collection.key()) @ UniversalNftError::InvalidTokenId
    )]
    pub nft_mint: Account<'info, Mint>,

    /// NFT token account (must be owned by sender and contain 1 token)
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = sender,
        constraint = nft_token_account.amount == 1 @ UniversalNftError::TokenDoesNotExist
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// NFT metadata account
    #[account(mut)]
    /// CHECK: Metaplex metadata account
    pub nft_metadata: UncheckedAccount<'info>,

    /// Sender (NFT owner) account
    #[account(mut)]
    pub sender: Signer<'info>,

    /// ZetaChain Gateway program
    /// CHECK: Gateway program for cross-chain calls
    #[account(address = ZETACHAIN_GATEWAY_PROGRAM_ID)]
    pub gateway: UncheckedAccount<'info>,

    /// Gateway PDA account (derived with proper seeds)
    #[account(
        mut,
        seeds = [GATEWAY_PDA_SEED],
        bump
    )]
    pub gateway_pda: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,
    
    /// Token program
    pub token_program: Program<'info, Token>,
    
    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// Metaplex token metadata program
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}
