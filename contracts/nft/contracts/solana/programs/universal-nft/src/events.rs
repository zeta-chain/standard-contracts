use anchor_lang::prelude::*;

#[event]
pub struct CollectionInitialized {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub tss_address: [u8; 20],
}

#[event]
pub struct TokenMinted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub name: String,
    pub uri: String,
    pub origin_chain: u64,
    pub is_solana_native: bool,
    pub timestamp: i64,
}

#[event]
pub struct TokenTransfer {
    pub collection: Pubkey,
    pub token_id: u64,
    pub destination_chain_id: u64,
    pub recipient: Vec<u8>,
    pub uri: String,
    pub sender: Pubkey,
    pub message: Vec<u8>,
    pub origin_mint: Pubkey,
    pub origin_chain: u64,
    pub is_returning: bool,
    pub timestamp: i64,
}

#[event]
pub struct TokenTransferReceived {
    pub collection: Pubkey,
    pub token_id: u64,
    pub recipient: Pubkey,
    pub uri: String,
    pub original_sender: Vec<u8>,
    pub nonce: u64,
    pub origin_chain: u64,
    pub original_mint: Option<Pubkey>,
    pub is_returning: bool,
    pub metadata_preserved: bool,
    pub timestamp: i64,
}

#[event]
pub struct TokenTransferReverted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub sender: Pubkey,
    pub uri: String,
    pub refund_amount: u64,
    pub origin_chain: u64,
    pub original_mint: Option<Pubkey>,
    pub revert_reason: String,
    pub timestamp: i64,
}

#[event]
pub struct SetUniversal {
    pub collection: Pubkey,
    pub universal_address: Pubkey,
}

#[event]
pub struct SetConnected {
    pub collection: Pubkey,
    pub chain_id: Vec<u8>,
    pub contract_address: Vec<u8>,
}

// New origin-specific events

#[event]
pub struct NftOriginCreated {
    #[index]
    pub token_id: u64,
    pub original_mint: Pubkey,
    #[index]
    pub collection: Pubkey,
    pub origin_chain: u64,
    pub metadata_uri: String,
    pub created_at: i64,
    pub bump: u8,
}

#[event]
pub struct NftOriginUpdated {
    #[index]
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub updated_fields: Vec<String>,
    pub new_metadata_uri: Option<String>,
    pub updated_at: i64,
}

#[event]
pub struct NftReturningToSolana {
    #[index]
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub new_mint: Pubkey,
    #[index]
    pub collection: Pubkey,
    pub metadata_preserved: bool,
    pub return_chain: u64,
    pub cycle_count: u32,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainCycleCompleted {
    #[index]
    pub token_id: u64,
    pub origin_chain: u64,
    pub destination_chain: u64,
    #[index]
    pub collection: Pubkey,
    pub cycle_count: u32,
    pub total_chains_visited: u32,
    pub cycle_duration: i64,
    pub timestamp: i64,
}

#[event]
pub struct OriginSystemStats {
    #[index]
    pub collection: Pubkey,
    pub total_origins_created: u64,
    pub solana_native_count: u64,
    pub cross_chain_count: u64,
    pub active_cycles: u32,
    pub timestamp: i64,
}

#[event]
pub struct MetadataPreservationEvent {
    #[index]
    pub token_id: u64,
    pub original_mint: Pubkey,
    pub preservation_status: String,
    pub metadata_hash: Option<String>,
    pub preservation_method: String,
    pub timestamp: i64,
}

#[event]
pub struct ChainMigrationEvent {
    #[index]
    pub token_id: u64,
    pub from_chain: u64,
    pub to_chain: u64,
    pub migration_type: String, // "outbound", "inbound", "return"
    pub origin_preserved: bool,
    pub metadata_updated: bool,
    pub timestamp: i64,
}

#[event]
pub struct OriginValidationEvent {
    #[index]
    pub token_id: u64,
    pub validation_status: String, // "success", "failed", "warning"
    pub validation_details: String,
    pub origin_chain_verified: bool,
    pub metadata_integrity_check: bool,
    pub timestamp: i64,
}
