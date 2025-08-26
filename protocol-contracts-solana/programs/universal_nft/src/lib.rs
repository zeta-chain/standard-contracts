use anchor_lang::prelude::*;

pub mod handle_incoming;
pub mod mint;
pub mod on_call;
pub mod state;
pub mod utils;



declare_id!("7bwYem3NvksZcsmgBLtNbQLkS1p35ahHa7JsssKeQ8UT");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("Initialized!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
