use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
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

    // Create nft_origin PDA to store metadata
    let (nft_origin_pda, nft_origin_bump) = derive_nft_origin_pda(&token_id);
    
    let nft_origin = NftOrigin {
        origin_chain: 0, // 0 for Solana
        origin_token_id: token_id,
        origin_mint: ctx.accounts.mint.key(),
        metadata_uri,
        created_at: clock.unix_timestamp,
        bump: nft_origin_bump,
    };

    // Create the nft_origin account
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &nft_origin_pda,
            Rent::get()?.minimum_balance(NftOrigin::LEN),
            NftOrigin::LEN as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[NftOrigin::SEED, &token_id, &[nft_origin_bump]]],
    )?;

    // Initialize the nft_origin account with data
    let mut nft_origin_account = anchor_lang::solana_program::account_info::AccountInfo::try_from(&nft_origin_pda)?;
    let mut data = nft_origin_account.try_borrow_mut_data()?;
    nft_origin.try_serialize(&mut &mut data[..])?;

    msg!("Minted Universal NFT with token ID: {}", hex::encode(&token_id[..8]));
    msg!("Metadata URI: {}", nft_origin.metadata_uri);
    msg!("NFT Origin PDA: {}", nft_origin_pda);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Metadata URI too long")]
    MetadataTooLong,
}
