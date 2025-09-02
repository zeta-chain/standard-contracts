use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::{Collection, Creator, DataV2},
        CreateMetadataAccountsV3, Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::{constants::*, errors::*, state::*, utils::*};

pub fn mint_from_cross_chain(
    ctx: Context<MintFromCrossChain>,
    source_chain_id: u64,
    source_token_id: Vec<u8>,
    original_owner: Vec<u8>,
    metadata: CrossChainNftMetadata,
    signature: [u8; 64],
    recovery_id: u8,
) -> Result<()> {
    // Extract values from program_config early to avoid borrow conflicts
    let is_initialized;
    let current_nonce;
    let tss_address;
    let program_config_bump;
    let collection_mint;
    {
        let program_config = &ctx.accounts.program_config;
        is_initialized = program_config.is_initialized;
        current_nonce = program_config.nonce;
        tss_address = program_config.tss_address;
        program_config_bump = program_config.bump;
        collection_mint = program_config.collection_mint;
    }
    
    let nft_state = &mut ctx.accounts.nft_state;
    let gateway_message = &mut ctx.accounts.gateway_message;
    let clock = Clock::get()?;
    
    require!(
        is_initialized,
        UniversalNftError::ProgramNotInitialized
    );
    
    require!(
        source_chain_id != SOLANA_CHAIN_ID,
        UniversalNftError::InvalidChainId
    );
    
    require!(
        !gateway_message.processed,
        UniversalNftError::MessageAlreadyProcessed
    );
    
    validate_metadata(&metadata)?;

    // Verify TSS signature
    let message_type = CrossChainMessageType::MintRequest {
        recipient: ctx.accounts.recipient.key(),
        metadata: metadata.clone(),
    };
    
    let message_hash = create_cross_chain_message_hash(
        SOLANA_CHAIN_ID,
        current_nonce,
        &message_type,
    )?;
    
    verify_tss_signature(
        &message_hash,
        &signature,
        recovery_id,
        &tss_address,
    )?;

    // Generate new token ID for Solana
    let token_id = generate_unique_token_id(&ctx.accounts.nft_mint.key(), &clock)?;
    
    // Initialize NFT state
    nft_state.mint = ctx.accounts.nft_mint.key();
    nft_state.original_owner = ctx.accounts.recipient.key();
    nft_state.token_id = token_id;
    nft_state.creation_timestamp = clock.unix_timestamp;
    nft_state.creation_slot = clock.slot;
    nft_state.chain_origin = metadata.original_chain_id;
    nft_state.is_cross_chain_locked = false;
    nft_state.metadata_hash = calculate_metadata_hash(&metadata)?;
    nft_state.bump = ctx.bumps.nft_state;

    // Record the inbound transfer
    let cross_chain_transfer = CrossChainTransfer {
        destination_chain_id: SOLANA_CHAIN_ID,
        destination_address: ctx.accounts.recipient.key().to_bytes().to_vec(),
        transfer_timestamp: clock.unix_timestamp,
        transaction_hash: message_hash,
        transfer_type: TransferType::Inbound,
    };
    nft_state.cross_chain_history = vec![cross_chain_transfer];

    // Mint the NFT to recipient
    let seeds = &[
        PROGRAM_SEED,
        &[program_config_bump],
    ];
    let signer = &[&seeds[..]];
    
    let mint_to_cpi_accounts = MintTo {
        mint: ctx.accounts.nft_mint.to_account_info(),
        to: ctx.accounts.nft_token_account.to_account_info(),
        authority: ctx.accounts.program_config.to_account_info(),
    };
    let mint_to_cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        mint_to_cpi_accounts,
        signer,
    );
    mint_to(mint_to_cpi_context, NFT_SUPPLY)?;

    // Prepare metadata before using program_config mutably
    let program_config_key = ctx.accounts.program_config.key();
    
    // Create metadata for the cross-chain NFT
    let creators = vec![Creator {
        address: program_config_key,
        verified: true,
        share: 100,
    }];

    let collection = Collection {
        verified: false,
        key: collection_mint,
    };

    let data = DataV2 {
        name: metadata.name.clone(),
        symbol: metadata.symbol.clone(),
        uri: metadata.uri.clone(),
        seller_fee_basis_points: 0,
        creators: Some(creators),
        collection: Some(collection),
        uses: None,
    };

    let create_metadata_accounts_v3_cpi_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.nft_metadata.to_account_info(),
        mint: ctx.accounts.nft_mint.to_account_info(),
        mint_authority: ctx.accounts.program_config.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        update_authority: ctx.accounts.program_config.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let create_metadata_accounts_v3_cpi_context = CpiContext::new_with_signer(
        ctx.accounts.metadata_program.to_account_info(),
        create_metadata_accounts_v3_cpi_accounts,
        signer,
    );

    create_metadata_accounts_v3(
        create_metadata_accounts_v3_cpi_context,
        data,
        true, // is_mutable
        false, // update_authority_is_signer (program is authority)
        None,
    )?;

    // Mark gateway message as processed
    gateway_message.processed = true;
    gateway_message.timestamp = clock.unix_timestamp;

    // Update program statistics
    let program_config = &mut ctx.accounts.program_config;
    program_config.total_nfts_minted = safe_add_u64(
        program_config.total_nfts_minted,
        1
    )?;
    program_config.total_cross_chain_transfers = safe_add_u64(
        program_config.total_cross_chain_transfers,
        1
    )?;
    program_config.nonce = safe_add_u64(current_nonce, 1)?;

    msg!("Cross-chain NFT minted successfully");
    msg!("Token ID: {}", token_id);
    msg!("Source Chain: {}", source_chain_id);
    msg!("Original Token ID: {:?}", source_token_id);
    msg!("Recipient: {}", ctx.accounts.recipient.key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    source_chain_id: u64,
    source_token_id: Vec<u8>,
    original_owner: Vec<u8>,
    metadata: CrossChainNftMetadata,
)]
pub struct MintFromCrossChain<'info> {
    #[account(
        mut,
        seeds = [PROGRAM_SEED],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        init,
        payer = payer,
        space = NftState::calculate_len(1),
        seeds = [NFT_STATE_SEED, nft_mint.key().as_ref()],
        bump
    )]
    pub nft_state: Account<'info, NftState>,

    #[account(
        init,
        payer = payer,
        space = GatewayMessage::LEN,
        seeds = [
            GATEWAY_MESSAGE_SEED,
            &source_chain_id.to_le_bytes(),
            &program_config.nonce.to_le_bytes(),
        ],
        bump
    )]
    pub gateway_message: Account<'info, GatewayMessage>,

    #[account(
        init,
        payer = payer,
        mint::decimals = NFT_DECIMALS,
        mint::authority = program_config,
        mint::freeze_authority = program_config,
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: This account will be initialized by the metadata program
    #[account(
        mut,
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub nft_metadata: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is the recipient of the cross-chain NFT
    pub recipient: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}