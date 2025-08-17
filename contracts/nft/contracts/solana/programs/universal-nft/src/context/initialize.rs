use anchor_lang::prelude::*;
use crate::state::UniversalNftConfig;
use crate::util::constants::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = UNIVERSAL_NFT_CONFIG_SPACE,
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: ZetaChain gateway program
    pub gateway_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}
