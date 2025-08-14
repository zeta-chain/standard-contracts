use anchor_lang::prelude::*;
use crate::state::gateway::GatewayConfig;

#[derive(Accounts)]
pub struct InitializeGateway<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: PDA derived inside
    #[account(mut)]
    pub gateway_config: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeGateway>, gateway_program: Pubkey) -> Result<()> {
    let (pda, bump) = Pubkey::find_program_address(&[GatewayConfig::SEED], &crate::ID);
    require_keys_eq!(ctx.accounts.gateway_config.key(), pda, ErrorCode::GatewayConfigPdaMismatch);

    if ctx.accounts.gateway_config.data_is_empty() {
        let space = 8 + GatewayConfig::LEN;
        let lamports = Rent::get()?.minimum_balance(space);
        anchor_lang::solana_program::program::invoke_signed(
            &anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.authority.key(),
                &pda,
                lamports,
                space as u64,
                &crate::ID,
            ),
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.gateway_config.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[GatewayConfig::SEED, &[bump]]],
        )?;
    }

    let mut data = ctx.accounts.gateway_config.try_borrow_mut_data()?;
    let cfg = GatewayConfig { gateway_program, bump };
    cfg.try_serialize(&mut &mut data[..])?;
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Gateway config PDA mismatch")] GatewayConfigPdaMismatch,
}


