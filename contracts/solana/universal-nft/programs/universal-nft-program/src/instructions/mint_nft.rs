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

pub fn mint_nft(
    ctx: Context<MintNft>,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
) -> Result<()> {
    // Extract values from program_config early to avoid borrow conflicts
    let is_initialized;
    let collection_mint;
    let authority;
    {
        let program_config = &ctx.accounts.program_config;
        is_initialized = program_config.is_initialized;
        collection_mint = program_config.collection_mint;
        authority = program_config.authority;
    }
    
    let nft_state = &mut ctx.accounts.nft_state;
    let clock = Clock::get()?;
    
    require!(
        is_initialized,
        UniversalNftError::ProgramNotInitialized
    );
    
    // Validate that owner is authorized to mint (program authority check)
    require!(
        authority == ctx.accounts.owner.key(),
        UniversalNftError::InvalidAuthority
    );
    
    require!(
        name.len() <= MAX_NAME_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        symbol.len() <= MAX_SYMBOL_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        uri.len() <= MAX_URI_LENGTH,
        UniversalNftError::MetadataTooLong
    );

    // Generate unique token ID
    let token_id = generate_unique_token_id(&ctx.accounts.nft_mint.key(), &clock)?;
    
    // Initialize NFT state
    nft_state.mint = ctx.accounts.nft_mint.key();
    nft_state.original_owner = ctx.accounts.owner.key();
    nft_state.token_id = token_id;
    nft_state.creation_timestamp = clock.unix_timestamp;
    nft_state.creation_slot = clock.slot;
    nft_state.chain_origin = SOLANA_CHAIN_ID;
    nft_state.cross_chain_history = Vec::new();
    nft_state.is_cross_chain_locked = false;
    nft_state.metadata_hash = [0u8; 32]; // Will be updated after metadata creation
    nft_state.bump = ctx.bumps.nft_state;

    // Mint the NFT
    let mint_to_cpi_accounts = MintTo {
        mint: ctx.accounts.nft_mint.to_account_info(),
        to: ctx.accounts.nft_token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let mint_to_cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        mint_to_cpi_accounts,
    );
    mint_to(mint_to_cpi_context, NFT_SUPPLY)?;

    // Prepare creators list
    let mut final_creators = creators.unwrap_or_default();
    if final_creators.is_empty() {
        final_creators.push(Creator {
            address: ctx.accounts.owner.key(),
            verified: true,
            share: 100,
        });
    } else {
        require!(
            final_creators.len() <= MAX_CREATOR_COUNT,
            UniversalNftError::CreatorVerificationFailed
        );
    }

    // Create NFT metadata with collection
    let collection = Collection {
        verified: false, // Will be verified separately if needed
        key: collection_mint,
    };

    let data = DataV2 {
        name: name.clone(),
        symbol,
        uri: uri.clone(),
        seller_fee_basis_points: 0,
        creators: Some(final_creators),
        collection: Some(collection),
        uses: None,
    };

    let create_metadata_accounts_v3_cpi_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.nft_metadata.to_account_info(),
        mint: ctx.accounts.nft_mint.to_account_info(),
        mint_authority: ctx.accounts.owner.to_account_info(),
        payer: ctx.accounts.owner.to_account_info(),
        update_authority: ctx.accounts.owner.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let create_metadata_accounts_v3_cpi_context = CpiContext::new(
        ctx.accounts.metadata_program.to_account_info(),
        create_metadata_accounts_v3_cpi_accounts,
    );

    create_metadata_accounts_v3(
        create_metadata_accounts_v3_cpi_context,
        data,
        true, // is_mutable
        true, // update_authority_is_signer
        None, // collection_details
    )?;

    // Calculate and store metadata hash
    let cross_chain_metadata = CrossChainNftMetadata {
        name,
        symbol: ctx.accounts.nft_mint.key().to_string()[..10].to_string(),
        uri,
        original_chain_id: SOLANA_CHAIN_ID,
        original_token_id: token_id.to_le_bytes().to_vec(),
        original_creator: ctx.accounts.owner.key().to_bytes().to_vec(),
        attributes: Vec::new(), // Can be extended later
    };
    nft_state.metadata_hash = calculate_metadata_hash(&cross_chain_metadata)?;

    // Update program statistics
    let program_config = &mut ctx.accounts.program_config;
    program_config.total_nfts_minted = safe_add_u64(
        program_config.total_nfts_minted, 
        1
    )?;

    msg!("NFT minted successfully");
    msg!("Token ID: {}", token_id);
    msg!("Mint: {}", ctx.accounts.nft_mint.key());
    msg!("Owner: {}", ctx.accounts.owner.key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct MintNft<'info> {
    #[account(
        mut,
        seeds = [PROGRAM_SEED],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        init,
        payer = owner,
        space = NftState::calculate_len(0),
        seeds = [NFT_STATE_SEED, nft_mint.key().as_ref()],
        bump
    )]
    pub nft_state: Account<'info, NftState>,

    #[account(
        init,
        payer = owner,
        mint::decimals = NFT_DECIMALS,
        mint::authority = owner,
        mint::freeze_authority = owner,
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
        payer = owner,
        associated_token::mint = nft_mint,
        associated_token::authority = owner,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}