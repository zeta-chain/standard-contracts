use anchor_lang::prelude::*;
use crate::state::UniversalNftConfig;
use crate::util::constants::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    pub authority: Signer<'info>,
}
