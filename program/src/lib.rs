use anchor_lang::prelude::*;

declare_id!("EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU");

#[program]
pub mod zetamint {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("ZetaMint program initialized!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
