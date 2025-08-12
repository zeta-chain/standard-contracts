use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::instruction as mpl_ix;
use mpl_token_metadata::ID as MPL_ID;

use crate::state::nft_origin::NftOrigin;
use crate::state::nft_origin::CrossChainNftPayload;
use crate::utils::derive_nft_origin_pda;

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
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(_ctx: Context<MintNewNft>, _metadata_uri: String) -> Result<()> {
    Ok(())
}
