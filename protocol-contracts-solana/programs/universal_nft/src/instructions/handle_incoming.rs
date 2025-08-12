use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct HandleIncoming<'info> {}

pub fn handler(_ctx: Context<HandleIncoming>, _payload: Vec<u8>) -> Result<()> {
    Ok(())
}
