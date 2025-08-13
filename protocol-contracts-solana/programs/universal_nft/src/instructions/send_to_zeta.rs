use anchor_lang::prelude::*;
use crate::state::nft_origin::CrossChainNftPayload;

#[derive(Accounts)]
pub struct SendToZeta<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Gateway program account to CPI into
    pub gateway_program: UncheckedAccount<'info>,
}

// Stub to demonstrate constructing payload; actual CPI depends on gateway interface
pub fn handler(_ctx: Context<SendToZeta>, _payload: CrossChainNftPayload) -> Result<()> {
    // TODO: replace with actual CPI to ZetaChain Gateway program once interface is available
    Ok(())
}


