use anchor_lang::prelude::*;

declare_id!("2QDqQXRX1gUq5czLmAMM84WL4yVUiPQYuYKviof6Q2pQ");

mod state;
mod errors;
mod operations;
mod instructions;
mod util;
mod callbacks;

use instructions::*;
use callbacks::*;

#[program]
pub mod universal_nft {
    use super::*;

    /// Initializes the Universal NFT program.
    ///
    /// # Arguments
    /// * `ctx` - The instruction context.
    /// * `zeta_gateway_program_id` - The program ID of the gateway program.
    pub fn initialize(
        ctx: Context<Initialize>, zeta_gateway_program_id: Pubkey
    ) -> Result<()> {
        Initialize::init(ctx, zeta_gateway_program_id)
    }

    /// Modify program configuration settings
    /// Allows authorized administrator to update gateway settings, admin, and pause state
    ///
    /// # Arguments
    /// * `ctx` - The instruction context.
    /// * `new_admin` - Optional new administrator public key.
    /// * `new_gateway_id` - Optional new ZetaChain gateway program ID.
    /// * `new_verifier` - Optional new gateway verifier PDA.
    /// * `pause_state` - Optional pause/unpause program state.
    pub fn modify_program_settings(
        ctx: Context<ModifySettings>,
        new_admin: Option<Pubkey>,
        new_gateway_id: Option<Pubkey>,
        new_verifier: Option<Pubkey>,
        pause_state: Option<bool>,
    ) -> Result<()> {
        ModifySettings::modify_program_settings(
            ctx,
            new_admin,
            new_gateway_id,
            new_verifier,
            pause_state,
        )
    }

    /// Allocates a unique token identifier for a specific mint address.
    /// Creates a reservation account that can be used later for minting.
    ///
    /// # Arguments
    /// * `ctx` - The instruction context.
    pub fn allocate_token_id(ctx: Context<AllocateTokenId>) -> Result<()> {
        AllocateTokenId::allocate_token_identifier(ctx)
    }

    /// Mints a new Universal NFT on Solana
    /// Creates NFT, metadata, master edition, and origin tracking
    ///
    /// # Arguments
    /// * `ctx` - The instruction context.
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
    ///
    /// # Arguments
    /// * `ctx` - The instruction context.
    /// * `asset_identifier` - The unique identifier of the NFT asset to bridge
    /// * `zetachain_universal_contract` - The address of the universal contract on ZetaChain
    /// * `final_destination_chain` - The chain ID of the final destination chain
    /// * `final_recipient` - The recipient address on the destination chain
    /// * `sol_deposit_lamports` - Amount of SOL to deposit for cross-chain fees
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

    /// Handle cross-chain callback from ZetaChain
    /// Recovers NFT from cross-chain bridge and mints it back to Solana
    ///
    /// # Arguments
    /// * `ctx` - The instruction context.
    /// * `_sol_amount` - Amount of SOL sent with the cross-chain message (if any)
    /// * `_zetachain_contract` - The address of the ZetaChain universal contract that initiated the call
    /// * `encoded_data` - Encoded NFT data containing token ID, recipient, metadata URI, name, symbol, etc.
    pub fn on_cross_chain_call(
        ctx: Context<CrossChainCallback>,
        _sol_amount: u64,
        _zetachain_contract: [u8; 20],
        encoded_data: Vec<u8>,
    ) -> Result<()> {
        CrossChainCallback::handle_cross_chain_callback(
            ctx, 
            _sol_amount, 
            _zetachain_contract, 
            encoded_data
        )
    }
}

