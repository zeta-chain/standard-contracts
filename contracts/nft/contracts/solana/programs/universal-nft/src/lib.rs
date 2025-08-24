use anchor_lang::prelude::*;

declare_id!("GZVn56cKAKbq6K76jf2rigSktqELicPjvneNBvEgDBkM");

mod state;
mod errors;
mod operations;
mod instructions;
mod util;
mod transaction_logs;

use instructions::*;

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

    /// Allocates a unique token identifier for a specific mint address.
    /// Creates a reservation account that can be used later for minting.
    ///
    /// # Arguments
    /// * `ctx` - The context of the transaction.
    pub fn allocate_token_id(ctx: Context<AllocateTokenId>) -> Result<()> {
        AllocateTokenId::allocate_token_identifier(ctx)
    }

    /// Mints a new Universal NFT on Solana
    /// Creates NFT, metadata, master edition, and origin tracking
    ///
    /// # Arguments
    /// * `ctx` - The context of the transaction.
    /// * `name` - The name of the Universal NFT.
    /// * `symbol` - The symbol of the Universal NFT.
    /// * `uri` - The URI of the Universal NFT metadata.
    pub fn mint_universal_nft(
        ctx: Context<MintUniversalNft>,
        name: String,
        symbol: String,
        uri: String
    ) -> Result<()> {
        MintUniversalNft::mint_universal_nft(ctx, name, symbol, uri)
    }

    /// Bridge asset to ZetaChain via cross-chain bridge call
    /// Burns the asset on Solana and triggers the universal contract on ZetaChain
    pub fn bridge_to_zetachain(
        ctx: Context<CrossChainBridge>,
        asset_identifier: [u8; 32],
        zetachain_universal_contract: [u8; 20], 
        final_destination_chain: u64, 
        final_recipient: String, 
        sol_deposit_lamports: u64, 
    ) -> Result<()> {
        CrossChainBridge::bridge_to_zetachain(
            ctx, 
            asset_identifier, 
            zetachain_universal_contract, 
            final_destination_chain, 
            final_recipient, 
            sol_deposit_lamports
        )
    }

}

