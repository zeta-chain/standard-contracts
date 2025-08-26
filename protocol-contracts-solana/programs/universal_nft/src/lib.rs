use anchor_lang::prelude::*;

pub mod handle_incoming;
pub mod mint;
pub mod on_call;
pub mod state;
pub mod utils;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub fn mint_new_nft(ctx: Context<mint::MintNewNft>, metadata_uri: String) -> Result<()> {
    mint::handler(ctx, metadata_uri)
}

pub fn handle_incoming(ctx: Context<handle_incoming::HandleIncoming>, payload: Vec<u8>) -> Result<()> {
    handle_incoming::handler(ctx, payload)
}

pub fn on_call(ctx: Context<on_call::OnCall>, payload: Vec<u8>) -> Result<()> {
    on_call::handler(ctx, payload)
}
