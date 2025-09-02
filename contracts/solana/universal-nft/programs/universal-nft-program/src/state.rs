use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[account]
pub struct ProgramConfig {
    pub authority: Pubkey,
    pub gateway_program_id: Pubkey,
    pub tss_address: [u8; 20],
    pub collection_mint: Pubkey,
    pub collection_metadata: Pubkey,
    pub nonce: u64,
    pub total_nfts_minted: u64,
    pub total_cross_chain_transfers: u64,
    pub is_initialized: bool,
    pub bump: u8,
}

impl ProgramConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // gateway_program_id
        20 + // tss_address
        32 + // collection_mint
        32 + // collection_metadata
        8 +  // nonce
        8 +  // total_nfts_minted
        8 +  // total_cross_chain_transfers
        1 +  // is_initialized
        1 +  // bump
        100; // padding for future expansion
}

#[account]
pub struct NftState {
    pub mint: Pubkey,
    pub original_owner: Pubkey,
    pub token_id: u64,
    pub creation_timestamp: i64,
    pub creation_slot: u64,
    pub chain_origin: u64, 
    pub cross_chain_history: Vec<CrossChainTransfer>,
    pub is_cross_chain_locked: bool,
    pub metadata_hash: [u8; 32],
    pub bump: u8,
}

impl NftState {
    pub const BASE_LEN: usize = 8 + // discriminator
        32 + // mint
        32 + // original_owner
        8 +  // token_id
        8 +  // creation_timestamp
        8 +  // creation_slot
        8 +  // chain_origin
        4 +  // vec length for cross_chain_history
        1 +  // is_cross_chain_locked
        32 + // metadata_hash
        1 +  // bump
        50;  // padding

    pub const MAX_CROSS_CHAIN_HISTORY: usize = 10;
    
    pub fn calculate_len(history_count: usize) -> usize {
        Self::BASE_LEN + (history_count.min(Self::MAX_CROSS_CHAIN_HISTORY) * CrossChainTransfer::LEN)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainTransfer {
    pub destination_chain_id: u64,
    pub destination_address: Vec<u8>,
    pub transfer_timestamp: i64,
    pub transaction_hash: [u8; 32],
    pub transfer_type: TransferType,
}

impl CrossChainTransfer {
    pub const LEN: usize = 8 + // destination_chain_id
        4 + 64 + // destination_address (vec with max 64 bytes)
        8 +     // transfer_timestamp
        32 +    // transaction_hash
        1;      // transfer_type
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TransferType {
    Outbound,  // NFT burned on Solana, sent to another chain
    Inbound,   // NFT minted on Solana, received from another chain
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainNftMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub original_chain_id: u64,
    pub original_token_id: Vec<u8>,
    pub original_creator: Vec<u8>,
    pub attributes: Vec<NftAttribute>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftAttribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RevertContext {
    pub original_sender: [u8; 20],
    pub original_chain_id: u64,
    pub token_id: Vec<u8>,
    pub revert_message: String,
    pub revert_timestamp: i64,
}

#[account]
pub struct GatewayMessage {
    pub sender: [u8; 20],
    pub chain_id: u64,
    pub nonce: u64,
    pub message_hash: [u8; 32],
    pub processed: bool,
    pub timestamp: i64,
    pub bump: u8,
}

impl GatewayMessage {
    pub const LEN: usize = 8 + // discriminator
        20 + // sender
        8 +  // chain_id
        8 +  // nonce
        32 + // message_hash
        1 +  // processed
        8 +  // timestamp
        1 +  // bump
        20;  // padding
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum CrossChainMessageType {
    MintRequest {
        recipient: Pubkey,
        metadata: CrossChainNftMetadata,
    },
    BurnConfirmation {
        token_id: u64,
        burned_amount: u64,
    },
    RevertRequest {
        original_transaction: [u8; 32],
        revert_context: RevertContext,
    },
}

pub const SOLANA_CHAIN_ID: u64 = 7565164; 

pub const PROGRAM_SEED: &[u8] = b"universal_nft_program";
pub const NFT_STATE_SEED: &[u8] = b"nft_state";
pub const GATEWAY_MESSAGE_SEED: &[u8] = b"gateway_message";
pub const COLLECTION_SEED: &[u8] = b"collection";