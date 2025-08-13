use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod utils;

use instructions::{mint, burn, handle_incoming, setup_gateway};

declare_id!("B5o6iRGvxvC8hL5tUTjV8dmu4GwFpnfH5ZssAedNt2Yp");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn mint_new_nft(ctx: Context<mint::MintNewNft>, metadata_uri: String) -> Result<()> {
        mint::handler(ctx, metadata_uri)
    }

    pub fn burn_for_transfer(ctx: Context<burn::BurnForTransfer>, nonce: u64) -> Result<()> {
        burn::handler(ctx, nonce)
    }

    pub fn handle_incoming(ctx: Context<handle_incoming::HandleIncoming>, payload: Vec<u8>) -> Result<()> {
        handle_incoming::handler(ctx, payload)
    }

    pub fn initialize_gateway(ctx: Context<setup_gateway::InitializeGateway>, gateway_program: Pubkey) -> Result<()> {
        setup_gateway::handler(ctx, gateway_program)
    }
}
