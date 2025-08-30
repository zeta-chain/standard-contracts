use anchor_lang::prelude::*;

declare_id!("Gc1BJg4sYAYGnKBStAHLTdVRLR3fA7DPc7t9G7vjKa1i");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;
use state::*;

#[program]
pub mod universal_nft_program {
    use super::*;

    pub fn initialize_program(
        ctx: Context<InitializeProgram>,
        gateway_program_id: Pubkey,
        collection_name: String,
        collection_symbol: String,
        collection_uri: String,
    ) -> Result<()> {
        instructions::initialize_program(ctx, gateway_program_id, collection_name, collection_symbol, collection_uri)
    }

    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<anchor_spl::metadata::mpl_token_metadata::types::Creator>>,
    ) -> Result<()> {
        instructions::mint_nft(ctx, name, symbol, uri, creators)
    }

    pub fn burn_for_cross_chain(
        ctx: Context<BurnForCrossChain>,
        destination_chain_id: u64,
        destination_address: Vec<u8>,
    ) -> Result<()> {
        instructions::burn_for_cross_chain(ctx, destination_chain_id, destination_address)
    }

    pub fn mint_from_cross_chain(
        ctx: Context<MintFromCrossChain>,
        source_chain_id: u64,
        source_token_id: Vec<u8>,
        original_owner: Vec<u8>,
        metadata: CrossChainNftMetadata,
        signature: [u8; 64],
        recovery_id: u8,
    ) -> Result<()> {
        instructions::mint_from_cross_chain(
            ctx,
            source_chain_id,
            source_token_id,
            original_owner,
            metadata,
            signature,
            recovery_id,
        )
    }

    pub fn on_call(
        ctx: Context<OnCall>,
        sender: [u8; 20],
        message: Vec<u8>,
    ) -> Result<()> {
        instructions::on_call(ctx, sender, message)
    }

    pub fn on_revert(
        ctx: Context<OnRevert>,
        revert_context: RevertContext,
    ) -> Result<()> {
        instructions::on_revert(ctx, revert_context)
    }

    pub fn update_gateway_config(
        ctx: Context<UpdateGatewayConfig>,
        new_gateway_program_id: Option<Pubkey>,
        new_tss_address: Option<[u8; 20]>,
    ) -> Result<()> {
        instructions::update_gateway_config(ctx, new_gateway_program_id, new_tss_address)
    }
}