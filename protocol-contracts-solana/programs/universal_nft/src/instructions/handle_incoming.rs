use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::{
    instruction as mpl_instruction,
    state::{Creator, DataV2, Collection, CollectionDetails, Uses},
};
use sha2::{Digest, Sha256};

use crate::state::nft_origin::NftOrigin;
use crate::utils::*;

#[derive(Accounts)]
pub struct HandleIncoming<'info> {
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
    #[account(mut)]
    pub recipient_token_account: UncheckedAccount<'info>,
    /// CHECK: nft_origin PDA to store origin information
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,
    /// CHECK: gateway signer for cross-chain message verification
    pub gateway_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<HandleIncoming>, payload: Vec<u8>) -> Result<()> {
    let clock = Clock::get()?;
    
    // Verify gateway signer (this would be verified by the gateway contract)
    // For now, we'll assume the gateway has already verified the message
    
    // Deserialize the cross-chain payload
    let cross_chain_payload: CrossChainNftPayload = CrossChainNftPayload::try_from_slice(&payload)
        .map_err(|_| ErrorCode::InvalidPayload)?;
    
    // Check if nft_origin already exists for this token_id
    let (nft_origin_pda, bump) = derive_nft_origin_pda(&cross_chain_payload.token_id);
    let nft_origin = &mut ctx.accounts.nft_origin;
    
    let mut is_existing_origin = false;
    let mut original_metadata_uri = cross_chain_payload.metadata_uri.clone();
    
    // Try to read existing nft_origin
    if !nft_origin.data_is_empty() {
        let existing_origin = NftOrigin::try_from(&**nft_origin)?;
        if existing_origin.origin_token_id == cross_chain_payload.token_id {
            is_existing_origin = true;
            original_metadata_uri = existing_origin.metadata_uri;
        }
    }
    
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
                system_program: ctx.accounts.payer.to_account_info(),
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
        name: format!("Universal NFT #{}", hex::encode(&cross_chain_payload.token_id[..8])),
        symbol: "UNFT",
        uri: original_metadata_uri.clone(),
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
        original_metadata_uri.clone(),
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
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
        ],
    )?;
    
    // Update or create nft_origin PDA
    let nft_origin_data = NftOrigin {
        origin_chain: cross_chain_payload.origin_chain_id,
        origin_token_id: cross_chain_payload.token_id.clone(),
        origin_mint: cross_chain_payload.origin_mint,
        metadata_uri: original_metadata_uri,
        created_at: clock.unix_timestamp,
        bump,
    };
    
    nft_origin.assign(&nft_origin_pda);
    nft_origin.set_inner(nft_origin_data);
    
    // Emit cross-chain mint event
    emit!(CrossChainMintEvent {
        token_id: cross_chain_payload.token_id,
        origin_chain: cross_chain_payload.origin_chain_id,
        new_mint: ctx.accounts.mint.key(),
        recipient: cross_chain_payload.recipient,
        is_existing_origin,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("✅ Cross-chain NFT minted successfully!");
    msg!("Token ID: {}", hex::encode(&cross_chain_payload.token_id));
    msg!("Origin Chain: {}", cross_chain_payload.origin_chain_id);
    msg!("New Mint: {}", ctx.accounts.mint.key());
    msg!("Is Existing Origin: {}", is_existing_origin);
    
    Ok(())
}

#[event]
pub struct CrossChainMintEvent {
    pub token_id: Vec<u8>,
    pub origin_chain: u16,
    pub new_mint: Pubkey,
    pub recipient: Pubkey,
    pub is_existing_origin: bool,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid payload format")]
    InvalidPayload,
    #[msg("Invalid NFT origin account")]
    InvalidNftOriginAccount,
    #[msg("Failed to create metadata")]
    MetadataCreationFailed,
    #[msg("Failed to create master edition")]
    MasterEditionCreationFailed,
}

// Cross-chain payload structure (same as in burn.rs)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrossChainNftPayload {
    pub version: u8,
    pub token_id: Vec<u8>,
    pub origin_chain_id: u16,
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub recipient: Pubkey,
    pub nonce: u64,
}
