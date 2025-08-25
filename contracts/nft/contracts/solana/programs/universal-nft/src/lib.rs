use anchor_lang::prelude::*;

declare_id!("GZVn56cKAKbq6K76jf2rigSktqELicPjvneNBvEgDBkM");

mod state;
mod errors;
mod instructions;
use instructions::init::*;


#[program]
pub mod universal_nft {
    use super::*;

    /// Initializes the Universal NFT program.
    ///
    /// # Arguments
    /// * `ctx` - The context of the transaction.
    /// * `zeta_gateway_program_id` - The program ID of the gateway program.
    pub fn initialize(
        ctx: Context<Initialize>, zeta_gateway_program_id: Pubkey
    ) -> Result<()> {
        Initialize::init(ctx, zeta_gateway_program_id)
    }

}

