use anchor_lang::prelude::*;
use crate::state::{UniversalNftConfig, MintTicket};
use crate::util::constants::*;

#[derive(Accounts)]
pub struct ReserveNextTokenId<'info> {
    #[account(
        mut,
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, UniversalNftConfig>,

    /// Ticket PDA to be created
    #[account(
        init,
        payer = authority,
        space = 8 + MintTicket::LEN,
        seeds = [MINT_TICKET_SEED, mint.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub ticket: Account<'info, MintTicket>,

    /// CHECK: only used as seed
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
