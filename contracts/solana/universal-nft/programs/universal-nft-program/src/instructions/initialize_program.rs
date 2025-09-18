use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::{CollectionDetails, Creator, DataV2},
        CreateMetadataAccountsV3, Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
// Removed unused imports

use crate::{constants::*, errors::*, state::*};

pub fn initialize_program(
    ctx: Context<InitializeProgram>,
    gateway_program_id: Pubkey,
    collection_name: String,
    collection_symbol: String,
    collection_uri: String,
) -> Result<()> {
    let program_config = &mut ctx.accounts.program_config;
    
    require!(
        !program_config.is_initialized,
        UniversalNftError::ProgramAlreadyInitialized
    );
    
    require!(
        collection_name.len() <= MAX_NAME_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        collection_symbol.len() <= MAX_SYMBOL_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        collection_uri.len() <= MAX_URI_LENGTH,
        UniversalNftError::MetadataTooLong
    );

    // Validate gateway program ID is not default
    require!(
        gateway_program_id != Pubkey::default(),
        UniversalNftError::InvalidGatewayProgramId
    );
    
    program_config.authority = ctx.accounts.authority.key();
    program_config.gateway_program_id = gateway_program_id;
    program_config.tss_address = [0u8; 20]; // Will be set later
    program_config.collection_mint = ctx.accounts.collection_mint.key();
    program_config.collection_metadata = ctx.accounts.collection_metadata.key();
    program_config.nonce = 1;
    program_config.total_nfts_minted = 0;
    program_config.total_cross_chain_transfers = 0;
    program_config.is_initialized = true;
    program_config.bump = ctx.bumps.program_config;

    // Mint collection NFT
    let mint_to_cpi_accounts = MintTo {
        mint: ctx.accounts.collection_mint.to_account_info(),
        to: ctx.accounts.collection_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let mint_to_cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        mint_to_cpi_accounts,
    );
    mint_to(mint_to_cpi_context, NFT_SUPPLY)?;

    // Create collection metadata
    let creators = vec![Creator {
        address: ctx.accounts.authority.key(),
        verified: true,
        share: 100,
    }];
    
    let data = DataV2 {
        name: collection_name,
        symbol: collection_symbol,
        uri: collection_uri,
        seller_fee_basis_points: 0,
        creators: Some(creators),
        collection: None,
        uses: None,
    };

    let create_metadata_accounts_v3_cpi_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.collection_metadata.to_account_info(),
        mint: ctx.accounts.collection_mint.to_account_info(),
        mint_authority: ctx.accounts.authority.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        update_authority: ctx.accounts.authority.to_account_info(),
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
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    msg!("Universal NFT Program initialized successfully");
    msg!("Collection mint: {}", ctx.accounts.collection_mint.key());
    msg!("Gateway program: {}", gateway_program_id);
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(gateway_program_id: Pubkey, collection_name: String)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = authority,
        space = ProgramConfig::LEN,
        seeds = [PROGRAM_SEED],
        bump
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        init,
        payer = authority,
        mint::decimals = NFT_DECIMALS,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK: This account will be initialized by the metadata program
    #[account(
        mut,
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            collection_mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub collection_metadata: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = collection_mint,
        associated_token::authority = authority,
    )]
    pub collection_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}