use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    clock::Clock,
    keccak,
    program::invoke_signed,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

// Import proper Metaplex CPI modules
use mpl_token_metadata::{
    ID as TOKEN_METADATA_PROGRAM_ID,
    instructions::{
        CreateMetadataAccountV3,
        CreateMetadataAccountV3InstructionArgs,
        CreateMasterEditionV3,
        CreateMasterEditionV3InstructionArgs,
    },
    types::{DataV2, Creator, Collection as MetaplexCollection, Uses, CollectionDetails},
};

use crate::state::{Collection, NftOrigin};
use crate::UniversalNftError;

/// Enhanced mint_nft function that implements the NFT Origin system for new Solana mints
pub fn mint_nft(
    ctx: Context<MintNft>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    // Validate inputs
    require!(name.len() > 0 && name.len() <= 32, UniversalNftError::InvalidMessage);
    require!(symbol.len() > 0 && symbol.len() <= 10, UniversalNftError::InvalidMessage);
    require!(uri.len() > 0 && uri.len() <= 200, UniversalNftError::InvalidMessage);

    // Extract values before mutable borrow
    let collection = &ctx.accounts.collection;
    let collection_key = collection.key();
    let collection_authority = collection.authority;
    let collection_name = collection.name.clone();
    let collection_bump = collection.bump;
    let mint_pubkey = ctx.accounts.nft_mint.key();
    
    // Enforce authority
    require_keys_eq!(
        ctx.accounts.authority.key(),
        collection_authority,
        UniversalNftError::InvalidSignature
    );

    // Get current clock for deterministic token ID generation
    let clock = Clock::get()?;
    let current_slot = clock.slot;
    
    // Generate deterministic token_id using [mint_pubkey + block.number + next_token_id]
    let next_token_id = collection.next_token_id;
    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(&mint_pubkey.to_bytes());
    hash_input.extend_from_slice(&current_slot.to_le_bytes());
    hash_input.extend_from_slice(&next_token_id.to_le_bytes());
    
    let hash = keccak::hash(&hash_input);
    let token_id = u64::from_le_bytes([
        hash.0[0], hash.0[1], hash.0[2], hash.0[3],
        hash.0[4], hash.0[5], hash.0[6], hash.0[7]
    ]);

    // Validate token_id uniqueness by checking if origin PDA already exists
    let nft_origin_info = ctx.accounts.nft_origin.to_account_info();
    require!(
        nft_origin_info.data_is_empty(),
        UniversalNftError::InvalidTokenId
    );

    // Initialize NFT Origin data with collection's next token ID
    let token_id = collection.next_token_id;
    let nft_origin = &mut ctx.accounts.nft_origin;
    nft_origin.token_id = token_id;
    nft_origin.collection = collection_key;
    nft_origin.chain_of_origin = 103; // Solana devnet - adjust based on network
    nft_origin.created_at = clock.unix_timestamp;
    nft_origin.metadata_uri = uri.clone();
    nft_origin.bump = ctx.bumps.nft_origin;

    // Mint NFT token
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

    mint_to(cpi_ctx.with_signer(signer_seeds), 1)?;

    // Create proper Metaplex metadata using official CPI
    create_metadata_account_v3(
        &ctx.accounts.nft_metadata.to_account_info(),
        &ctx.accounts.nft_mint.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.authority.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.metadata_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        name.clone(),
        symbol.clone(),
        uri.clone(),
        signer_seeds,
    )?;

    // Create master edition for NFT uniqueness using official CPI
    create_master_edition_v3(
        &ctx.accounts.master_edition.to_account_info(),
        &ctx.accounts.nft_mint.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.authority.to_account_info(),
        &ctx.accounts.nft_metadata.to_account_info(),
        &ctx.accounts.metadata_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        signer_seeds,
    )?;

    // Update collection statistics
    let collection = &mut ctx.accounts.collection;
    collection.increment_total_minted()?;
    collection.increment_solana_native_count()?;
    collection.next_token_id = collection
        .next_token_id
        .checked_add(1)
        .ok_or(error!(UniversalNftError::InvalidTokenId))?;

    emit!(crate::TokenMinted {
        collection: collection_key,
        token_id,
        mint: mint_pubkey,
        recipient: ctx.accounts.recipient.key(),
        name,
        uri: uri.clone(),
        origin_chain: 103, // Solana devnet
        is_solana_native: true,
    });

    emit!(crate::NftOriginCreated {
        token_id,
        original_mint: mint_pubkey,
        collection: collection_key,
        origin_chain: 103,
        metadata_uri: uri.clone(),
    });

    Ok(())
}

/// Create Metaplex metadata account using proper CPI to mpl-token-metadata
fn create_metadata_account_v3<'a>(
    metadata_account: &AccountInfo<'a>,
    mint_account: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    name: String,
    symbol: String,
    uri: String,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Create DataV2 struct with metadata information
    let data = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0, // No royalties for Universal NFT
        creators: Some(vec![Creator {
            address: *update_authority.key,
            verified: true,
            share: 100,
        }]),
        collection: None, // Collection verification handled separately if needed
        uses: None,
    };

    // Create the instruction using official Metaplex instruction builder
    let create_metadata_ix = CreateMetadataAccountV3 {
        metadata: *metadata_account.key,
        mint: *mint_account.key,
        mint_authority: *mint_authority.key,
        payer: *payer.key,
        update_authority: *update_authority.key,
        system_program: *system_program.key,
        rent: Some(*rent.key),
    };

    let instruction_args = CreateMetadataAccountV3InstructionArgs {
        data,
        is_mutable: true,
        collection_details: None,
    };

    // Build the instruction
    let instruction = create_metadata_ix.instruction(instruction_args);

    // Execute CPI with proper error handling
    invoke_signed(
        &instruction,
        &[
            metadata_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            update_authority.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to create metadata account: {:?}", e);
        UniversalNftError::InvalidMessage
    })?;

    Ok(())
}

/// Create master edition for NFT uniqueness using proper CPI to mpl-token-metadata
fn create_master_edition_v3<'a>(
    master_edition_account: &AccountInfo<'a>,
    mint_account: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    metadata_account: &AccountInfo<'a>,
    metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Create the instruction using official Metaplex instruction builder
    let create_master_edition_ix = CreateMasterEditionV3 {
        edition: *master_edition_account.key,
        mint: *mint_account.key,
        update_authority: *mint_authority.key,
        mint_authority: *mint_authority.key,
        payer: *payer.key,
        metadata: *metadata_account.key,
        token_program: spl_token::ID,
        system_program: *system_program.key,
        rent: Some(*rent.key),
    };

    let instruction_args = CreateMasterEditionV3InstructionArgs {
        max_supply: None, // Unlimited supply for Universal NFT
    };

    // Build the instruction
    let instruction = create_master_edition_ix.instruction(instruction_args);

    // Execute CPI with proper error handling
    invoke_signed(
        &instruction,
        &[
            master_edition_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            metadata_account.clone(),
            metadata_program.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to create master edition: {:?}", e);
        UniversalNftError::InvalidMessage
    })?;

    Ok(())
}

/// Derive metadata PDA for a given mint
#[allow(dead_code)]
pub fn derive_metadata_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.as_ref(),
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    )
}

/// Derive master edition PDA for a given mint
#[allow(dead_code)]
pub fn derive_master_edition_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.as_ref(),
            b"edition",
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    )
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct MintNft<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,
    
    /// Authority controlling the collection and paying for the transaction
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// NFT recipient account
    /// CHECK: Can be any valid Solana address
    pub recipient: UncheckedAccount<'info>,

    /// NFT Origin PDA to track original mint and metadata
    #[account(
        init,
        payer = authority,
        space = 8 + NftOrigin::INIT_SPACE,
        seeds = [b"nft_origin", &collection.next_token_id.to_le_bytes()[..]],
        bump
    )]
    pub nft_origin: Account<'info, NftOrigin>,

    /// Metaplex metadata account
    /// CHECK: Derived from mint address and validated by Metaplex program
    #[account(
        mut,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub nft_metadata: UncheckedAccount<'info>,

    /// Master edition account for NFT uniqueness
    /// CHECK: Derived from mint address and validated by Metaplex program
    #[account(
        mut,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            nft_mint.key().as_ref(),
            b"edition",
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program - validated by address constraint
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

/// Generate a token ID for PDA derivation (now using collection's next_token_id)
#[allow(dead_code)]
fn generate_temp_token_id(mint: &Pubkey, authority: &Pubkey) -> u64 {
    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(&mint.to_bytes());
    hash_input.extend_from_slice(&authority.to_bytes());
    
    let hash = hash(&hash_input);
    u64::from_le_bytes([
        hash.0[0], hash.0[1], hash.0[2], hash.0[3],
        hash.0[4], hash.0[5], hash.0[6], hash.0[7]
    ])
}

/// Verify collection for an NFT (optional utility function)
#[allow(dead_code)]
pub fn verify_collection_for_nft(
    collection_metadata: &AccountInfo,
    collection_mint: &AccountInfo,
    collection_authority: &AccountInfo,
    nft_metadata: &AccountInfo,
    metadata_program: &AccountInfo,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // This would implement collection verification using Metaplex CPI
    // For now, this is a placeholder for future collection verification
    msg!("Collection verification would be implemented here");
    Ok(())
}

/// Enhanced error handling for Metaplex operations
pub fn handle_metaplex_error(error: anchor_lang::error::Error) -> UniversalNftError {
    msg!("Metaplex operation failed: {:?}", error);
    match error {
        _ => UniversalNftError::InvalidMessage,
    }
}

/// Enhanced TokenMinted event with origin information
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

/// NFT Origin creation event
#[event]
pub struct NftOriginCreated {
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub collection: Pubkey,
    pub origin_chain: u64,
    pub metadata_uri: String,
}
