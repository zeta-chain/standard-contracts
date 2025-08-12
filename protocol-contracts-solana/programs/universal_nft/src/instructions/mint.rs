use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::{
    instruction as mpl_instruction,
    state::{Creator, Data, DataV2, Collection, CollectionDetails, Uses},
};
use sha2::{Digest, Sha256};

use crate::state::nft_origin::NftOrigin;
use crate::utils::*;

#[derive(Accounts)]
pub struct MintNewNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: recipient may be any account; ATA will be derived
    pub recipient: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: Signer<'info>,
    /// CHECK: metadata PDA created via CPI to mpl-token-metadata
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: master edition PDA created via CPI
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: ATA will be derived and created via CPI
    #[account(mut)]
    pub recipient_token_account: UncheckedAccount<'info>,
    /// CHECK: nft_origin PDA to store origin information
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<MintNewNft>, metadata_uri: String) -> Result<()> {
    let clock = Clock::get()?;
    
    // Generate unique token ID: hash of mint pubkey + slot + nonce
    let mut hasher = Sha256::new();
    hasher.update(ctx.accounts.mint.key().as_ref());
    hasher.update(&clock.slot.to_le_bytes());
    hasher.update(&clock.unix_timestamp.to_le_bytes());
    let token_id = hasher.finalize().to_vec();
    
    // Create associated token account for recipient
    let (ata, _) = Pubkey::find_program_address(
        &[
            ctx.accounts.recipient.key().as_ref(),
            ctx.accounts.token_program.key().as_ref(),
            ctx.accounts.mint.key().as_ref(),
        ],
        &ctx.accounts.associated_token_program.key(),
    );
    
    // Create the ATA
    anchor_spl::associated_token::create(
        CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.recipient.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
    )?;
    
    // Mint 1 token to the recipient
    anchor_spl::token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        1,
    )?;
    
    // Create metadata account
    let (metadata_pda, _) = derive_metadata_pda(&ctx.accounts.mint.key());
    let (master_edition_pda, _) = derive_master_edition_pda(&ctx.accounts.mint.key());
    
    // Each new universal NFT creates a separate collection
    let collection_mint = ctx.accounts.mint.key();
    let collection_metadata = metadata_pda;
    
    let data_v2 = DataV2 {
        name: format!("Universal NFT #{}", hex::encode(&token_id[..8])),
        symbol: "UNFT",
        uri: metadata_uri.clone(),
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator {
            address: ctx.accounts.payer.key(),
            verified: true,
            share: 100,
        }]),
        collection: Some(Collection {
            verified: true,
            key: collection_metadata,
        }),
        uses: None,
    };
    
    let create_metadata_account_ix = mpl_instruction::create_metadata_accounts_v3(
        ctx.accounts.metadata.to_account_info().key(),
        ctx.accounts.mint.to_account_info().key(),
        ctx.accounts.payer.to_account_info().key(),
        ctx.accounts.payer.to_account_info().key(),
        ctx.accounts.payer.to_account_info().key(),
        metadata_uri,
        Some(data_v2.name),
        Some(data_v2.symbol),
        Some(data_v2.uri),
        Some(vec![Creator {
            address: ctx.accounts.payer.key(),
            verified: true,
            share: 100,
        }]),
        data_v2.seller_fee_basis_points,
        true,
        true,
        Some(CollectionDetails::V1 { __: [0u8; 0] }),
        Some(Uses {
            total: 1,
            use_method: mpl_token_metadata::state::UseMethod::Single,
            remaining: 1,
        }),
        None,
    );
    
    // Execute the metadata creation
    anchor_lang::solana_program::program::invoke(
        &create_metadata_account_ix,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
        ],
    )?;
    
    // Create master edition
    let create_master_edition_ix = mpl_instruction::create_master_edition_v3(
        master_edition_pda,
        ctx.accounts.metadata.to_account_info().key(),
        ctx.accounts.mint.to_account_info().key(),
        ctx.accounts.payer.to_account_info().key(),
        ctx.accounts.payer.to_account_info().key(),
        ctx.accounts.payer.to_account_info().key(),
        Some(0), // Max supply: 0 means unlimited
    );
    
    // Execute the master edition creation
    anchor_lang::solana_program::program::invoke(
        &create_master_edition_ix,
        &[
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
        ],
    )?;
    
    // Store origin information in PDA
    let nft_origin = &mut ctx.accounts.nft_origin;
    let nft_origin_data = NftOrigin {
        origin_chain: 0, // 0 for Solana
        origin_token_id: token_id.clone(),
        origin_mint: ctx.accounts.mint.key(),
        metadata_uri,
        created_at: clock.unix_timestamp,
        bump: 0, // Will be set by the account creation
    };
    
    // Initialize the nft_origin account
    let (nft_origin_pda, bump) = derive_nft_origin_pda(&token_id);
    nft_origin.assign(&nft_origin_pda);
    nft_origin.set_inner(nft_origin_data);
    
    // Set the bump
    let mut nft_origin_data = nft_origin.try_borrow_mut_data()?;
    nft_origin_data[0] = bump;
    
    msg!("✅ Universal NFT minted successfully!");
    msg!("Token ID: {}", hex::encode(&token_id));
    msg!("Mint: {}", ctx.accounts.mint.key());
    msg!("Collection: {}", collection_mint);
    
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid NFT origin account")]
    InvalidNftOriginAccount,
    #[msg("Failed to create metadata")]
    MetadataCreationFailed,
    #[msg("Failed to create master edition")]
    MasterEditionCreationFailed,
}
