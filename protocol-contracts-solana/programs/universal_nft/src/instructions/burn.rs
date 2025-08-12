use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct BurnForTransfer<'info> {}

pub fn handler(_ctx: Context<BurnForTransfer>, _nonce: u64) -> Result<()> {
    Ok(())
}
