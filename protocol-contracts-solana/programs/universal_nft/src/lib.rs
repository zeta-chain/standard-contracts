use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zP9LnUZ");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn mint_new_nft(ctx: Context<MintNewNft>, metadata_uri: String) -> Result<()> {
        instructions::mint::handler(ctx, metadata_uri)
    }

    pub fn burn_for_transfer(ctx: Context<BurnForTransfer>, nonce: u64) -> Result<()> {
        instructions::burn::handler(ctx, nonce)
    }

    pub fn handle_incoming(ctx: Context<HandleIncoming>, payload: Vec<u8>) -> Result<()> {
        instructions::handle_incoming::handler(ctx, payload)
    }
}
