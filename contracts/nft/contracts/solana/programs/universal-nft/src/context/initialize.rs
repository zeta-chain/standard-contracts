use anchor_lang::prelude::*;
use crate::state::UniversalNftConfig;
use crate::util::constants::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
    // Allocate total space (discriminator + payload)
        space = UniversalNftConfig::SPACE,
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: ZetaChain gateway program (must be executable)
    pub gateway_program: UncheckedAccount<'info>,

    /// ZetaChain Gateway PDA owned by the gateway program (for consistency and validation)
    /// CHECK: We validate ownership against the provided gateway_program
    #[account(
        constraint = *gateway_pda.owner == gateway_program.key() @ crate::error::UniversalNftError::InvalidGatewayProgram
    )]
    pub gateway_pda: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}
