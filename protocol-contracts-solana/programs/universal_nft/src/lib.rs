use anchor_lang::prelude::*;

pub mod ix;
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

    // Mint a new NFT on Solana and create origin PDA
    pub fn mint_new_nft(ctx: Context<ix::mint::MintNewNft>, metadata_uri: String) -> Result<()> {
        ix::mint::handler(ctx, metadata_uri)
    }

    // Burn NFT for cross-chain transfer and emit payload event
    pub fn burn_for_transfer(ctx: Context<ix::burn::BurnForTransfer>, nonce: u64) -> Result<()> {
        ix::burn::handler(ctx, nonce)
    }

    // Handle an incoming cross-chain message to mint/restrore NFT
    pub fn handle_incoming(ctx: Context<ix::handle_incoming::HandleIncoming>, payload: Vec<u8>) -> Result<()> {
        ix::handle_incoming::handler(ctx, payload)
    }

    // Configure the gateway program PDA
    pub fn setup_gateway(ctx: Context<ix::setup_gateway::InitializeGateway>, gateway_program: Pubkey) -> Result<()> {
        ix::setup_gateway::handler(ctx, gateway_program)
    }

    // Entry called by ZetaChain Gateway (routes to handle_incoming)
    pub fn on_call(ctx: Context<ix::on_call::OnCall>, payload: Vec<u8>) -> Result<()> {
        ix::on_call::handler(ctx, payload)
    }

    // Placeholder to send payloads to Zeta (actual CPI to be implemented when iface is finalized)
    pub fn send_to_zeta(ctx: Context<ix::send_to_zeta::SendToZeta>, payload: state::nft_origin::CrossChainNftPayload) -> Result<()> {
        ix::send_to_zeta::handler(ctx, payload)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
