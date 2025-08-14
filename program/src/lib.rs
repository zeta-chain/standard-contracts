use anchor_lang::prelude::*;

declare_id!("EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU");

#[program]
pub mod zetamint {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}