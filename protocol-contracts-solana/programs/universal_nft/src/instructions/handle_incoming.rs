use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct HandleIncoming<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: gateway signer verified by program logic
    pub gateway_signer: UncheckedAccount<'info>,
    /// CHECK: recipient of the re-minted nft
    pub recipient: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(_ctx: Context<HandleIncoming>, _payload: Vec<u8>) -> Result<()> {
    // Implementation to be added in Phase 3
    Ok(())
}
