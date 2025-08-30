use anchor_lang::prelude::*;

use crate::{errors::*, state::*};

pub fn update_gateway_config(
    ctx: Context<UpdateGatewayConfig>,
    new_gateway_program_id: Option<Pubkey>,
    new_tss_address: Option<[u8; 20]>,
) -> Result<()> {
    let program_config = &mut ctx.accounts.program_config;
    
    require!(
        program_config.is_initialized,
        UniversalNftError::ProgramNotInitialized
    );
    
    if let Some(gateway_id) = new_gateway_program_id {
        program_config.gateway_program_id = gateway_id;
        msg!("Updated gateway program ID to: {}", gateway_id);
    }
    
    if let Some(tss_addr) = new_tss_address {
        program_config.tss_address = tss_addr;
        msg!("Updated TSS address to: {:?}", tss_addr);
    }
    
    msg!("Gateway configuration updated successfully");
    
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateGatewayConfig<'info> {
    #[account(
        mut,
        seeds = [crate::state::PROGRAM_SEED],
        bump = program_config.bump,
        has_one = authority @ UniversalNftError::InvalidAuthority,
    )]
    pub program_config: Account<'info, ProgramConfig>,
    
    pub authority: Signer<'info>,
}