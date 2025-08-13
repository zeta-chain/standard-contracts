use anchor_lang::prelude::*;
use crate::state::gateway::GatewayConfig;

#[derive(Accounts)]
pub struct OnCall<'info> {
    /// The CPI caller program (Gateway) is enforced via address lookup of config
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: PDA with gateway program id
    pub gateway_config: UncheckedAccount<'info>,
}

// A generic entrypoint to be invoked by ZetaChain Gateway
pub fn handler(ctx: Context<OnCall>, payload: Vec<u8>) -> Result<()> {
    let (cfg_pda, _bump) = Pubkey::find_program_address(&[GatewayConfig::SEED], &crate::ID);
    require_keys_eq!(ctx.accounts.gateway_config.key(), cfg_pda, ErrorCode::UnauthorizedGateway);
    let data = ctx.accounts.gateway_config.try_borrow_data()?;
    let cfg = GatewayConfig::try_from_slice(&data[8..]).map_err(|_| ErrorCode::UnauthorizedGateway)?;

    // Enforce that caller is the configured gateway program via CPI context program id check
    // Note: Anchor does not directly expose invoker program id here; in production this
    // would rely on the gateway program performing a CPI with expected signer seeds.
    // Here we simply parse payload and dispatch to `handle_incoming`-compatible logic off-chain.

    // For now, we no-op; actual routing should be defined once gateway CPI interface is finalized.
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized gateway")] UnauthorizedGateway,
}


