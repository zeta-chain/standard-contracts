use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MintNewNft<'info> {}

pub fn handler(_ctx: Context<MintNewNft>, _metadata_uri: String) -> Result<()> {
    Ok(())
}
