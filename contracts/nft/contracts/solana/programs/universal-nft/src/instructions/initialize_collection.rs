use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{initialize_mint, InitializeMint, Token},
};

use crate::state::Collection;
use crate::{CollectionInitialized, TOKEN_METADATA_PROGRAM_ID, UniversalNftError};
use mpl_token_metadata::types::{DataV2, Creator};

/// Initialize a new Universal NFT collection compatible with ZetaChain
#[allow(dead_code)]
pub fn initialize_collection(
    ctx: Context<InitializeCollection>,
    name: String,
    symbol: String,
    uri: String,
    tss_address: [u8; 20],
) -> Result<()> {
    let collection = &mut ctx.accounts.collection;
    // Enforce canonical limits (adjust to struct caps)
    require!(name.len() <= 32, UniversalNftError::InvalidNameLength);
    require!(symbol.len() <= 10, UniversalNftError::InvalidSymbolLength);
    require!(uri.len() <= 200, UniversalNftError::InvalidUriLength);
    
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

    // Create collection mint and metadata
    create_collection_mint_and_metadata(&ctx, &name, &symbol, &uri)?;

    emit!(CollectionInitialized {
        collection: collection_key,
        authority: ctx.accounts.authority.key(),
        name,
        symbol,
        tss_address,
    });

    Ok(())
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
        seeds = [b"collection", authority.key().as_ref(), collection_mint.key().as_ref()],
        bump
    )]
    pub collection: Account<'info, Collection>,

    /// CHECK: Collection mint - used as PDA seed
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

/// Create collection mint and metadata for the collection
fn create_collection_mint_and_metadata(
    ctx: &Context<InitializeCollection>,
    name: &str,
    symbol: &str,
    uri: &str,
) -> Result<()> {
    // Initialize the collection mint
    let cpi_accounts = InitializeMint {
        mint: ctx.accounts.collection_mint.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    initialize_mint(
        cpi_ctx,
        0, // decimals for NFT
        &ctx.accounts.authority.key(),
        Some(&ctx.accounts.authority.key()),
    )?;

    // Create collection metadata using Metaplex
    let data = DataV2 {
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator {
            address: ctx.accounts.authority.key(),
            verified: true,
            share: 100,
        }]),
        collection: None,
        uses: None,
    };

    // Create metadata account instruction
    let metadata_instruction = crate::utils::create_metadata_instruction(
        &ctx.accounts.metadata_program.key(),
        &ctx.accounts.collection_metadata.key(),
        &ctx.accounts.collection_mint.key(),
        &ctx.accounts.authority.key(),
        &ctx.accounts.authority.key(),
        &ctx.accounts.authority.key(),
        data,
        true,
        true,
    )?;

    // Execute metadata creation
    anchor_lang::solana_program::program::invoke_signed(
        &metadata_instruction,
        &[
            ctx.accounts.collection_metadata.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[],
    )?;

    Ok(())
}
