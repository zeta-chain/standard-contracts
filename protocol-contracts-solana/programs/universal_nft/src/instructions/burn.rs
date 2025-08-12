use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::nft_origin::{NftOrigin, CrossChainNftPayload};
use crate::state::replay::ReplayMarker;

#[derive(Accounts)]
pub struct BurnForTransfer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, constraint = owner_token_account.owner == owner.key(), has_one = mint)]
    pub owner_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    /// CHECK: nft_origin PDA derived in handler
    #[account(mut)]
    pub nft_origin_pda: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(_ctx: Context<BurnForTransfer>, _nonce: u64) -> Result<()> {
    // Placeholder; full logic after Anchor install available locally
    Ok(())
}
