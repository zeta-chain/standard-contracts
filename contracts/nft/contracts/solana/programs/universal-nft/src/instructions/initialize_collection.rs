use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
};

use crate::state::Collection;
use crate::{CollectionInitialized, TOKEN_METADATA_PROGRAM_ID, UniversalNftError};

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
