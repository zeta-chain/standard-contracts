use anchor_lang::prelude::*;

declare_id!("GZVn56cKAKbq6K76jf2rigSktqELicPjvneNBvEgDBkM");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
