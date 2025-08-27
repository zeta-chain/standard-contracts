pub mod gateway;
pub mod nft_origin;
pub mod replay;

use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use gateway::GatewayConfig;

#[derive(Accounts)]
pub struct InitializeGatewayConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: PDA will be created
    #[account(mut)]
    pub gateway_config: UncheckedAccount<'info>,
    /// CHECK: expected gateway program
    pub gateway_program: UncheckedAccount<'info>,
    /// CHECK: pinned gateway PDA
    pub gateway_pda: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_gateway_config(
    ctx: Context<InitializeGatewayConfig>,
) -> Result<()> {
    let (cfg_pda, bump) = Pubkey::find_program_address(&[GatewayConfig::SEED], &crate::ID);
    require_keys_eq!(ctx.accounts.gateway_config.key(), cfg_pda, ErrorCode::UnauthorizedGateway);
    // Create account with exact space
    let space = GatewayConfig::LEN;
    let lamports = Rent::get()?.minimum_balance(space);
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &cfg_pda,
            lamports,
            space as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.gateway_config.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[GatewayConfig::SEED, &[bump]]],
    )?;

    // Serialize config
    let mut data = ctx.accounts.gateway_config.try_borrow_mut_data()?;
    data[..8].copy_from_slice(&GatewayConfig::discriminator());
    let cfg = GatewayConfig {
        gateway_program: ctx.accounts.gateway_program.key(),
        gateway_pda: ctx.accounts.gateway_pda.key(),
        bump,
    };
    cfg.try_serialize(&mut &mut data[8..])?;
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized gateway")]
    UnauthorizedGateway,
}
