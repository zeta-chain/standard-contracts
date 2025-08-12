use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::{
    instruction as mpl_instruction,
    state::{Creator, DataV2},
};
use sha2::{Digest, Sha256};

use crate::state::nft_origin::NftOrigin;
use crate::utils::*;

#[derive(Accounts)]
pub struct MintNewNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub recipient: SystemAccount<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer,
        mint::freeze_authority = payer,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    /// CHECK: Metaplex metadata PDA (derived off-chain)
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: Metaplex master edition PDA (derived off-chain)
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<MintNewNft>, metadata_uri: String) -> Result<()> {
    let clock = Clock::get()?;

    require!(metadata_uri.len() <= NftOrigin::MAX_URI_LEN, ErrorCode::MetadataTooLong);

    // Generate unique token ID: hash of mint pubkey + slot + timestamp
    let mut hasher = Sha256::new();
    hasher.update(ctx.accounts.mint.key().as_ref());
    hasher.update(&clock.slot.to_le_bytes());
    hasher.update(&clock.unix_timestamp.to_le_bytes());
    let token_id_hash = hasher.finalize();
    let mut token_id: [u8; 32] = [0u8; 32];
    token_id.copy_from_slice(&token_id_hash[..32]);

    // Prepare metadata
    let data_v2 = DataV2 {
        name: format!("Universal NFT #{}", hex::encode(&token_id[..8])),
        symbol: "UNFT".to_string(),
        uri: metadata_uri.clone(),
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator {
            address: ctx.accounts.payer.key(),
            verified: true,
            share: 100,
        }]),
        collection: None,
        uses: None,
    };

    // Create metadata
    let ix_meta = mpl_instruction::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        ctx.accounts.metadata.key(),
        ctx.accounts.mint.key(),
        ctx.accounts.payer.key(),      // mint authority
        ctx.accounts.payer.key(),      // payer
        ctx.accounts.payer.key(),      // update authority
        data_v2,
        true,                          // is_mutable
        None,                          // collection details
        None,                          // uses
        None,                          // collection
    );
    anchor_lang::solana_program::program::invoke(
        &ix_meta,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    // Create master edition
    let ix_edition = mpl_instruction::create_master_edition_v3(
        mpl_token_metadata::ID,
        ctx.accounts.master_edition.key(),
        ctx.accounts.mint.key(),
        ctx.accounts.payer.key(),      // update authority
        ctx.accounts.payer.key(),      // mint authority
        ctx.accounts.payer.key(),      // payer
        ctx.accounts.metadata.key(),
        Some(0),
    );
    anchor_lang::solana_program::program::invoke(
        &ix_edition,
        &[
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    // Mint 1 token to recipient
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

    // Create nft_origin PDA (account creation only; serialization handled in follow-up)
    let (nft_origin_pda, bump) = derive_nft_origin_pda(&token_id);
    let space = 8 + NftOrigin::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &nft_origin_pda,
            lamports,
            space as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            AccountInfo::new_readonly(&nft_origin_pda, false),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[NftOrigin::SEED, &token_id, &[bump]]],
    )?;

    // Serialize NftOrigin data
    {
        let mut data = ctx.accounts.payer.to_account_info(); // placeholder to satisfy borrow rules
    }
    let mut account_info = AccountInfo::new(&nft_origin_pda, false, true, &mut [], &crate::ID, false, 0);
    let mut dst = account_info.try_borrow_mut_data()?;
    let origin = NftOrigin {
        origin_chain: 0,
        origin_token_id: token_id,
        origin_mint: ctx.accounts.mint.key(),
        metadata_uri: metadata_uri,
        created_at: clock.unix_timestamp,
        bump,
    };
    origin.try_serialize(&mut &mut dst[..])?;

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
    #[msg("Metadata URI too long")]
    MetadataTooLong,
}
