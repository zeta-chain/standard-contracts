use anchor_lang::prelude::*;

/// Event emitted when a digital asset is sent to another blockchain
#[event]
pub struct InterChainTransferStarted {
    /// Digital asset identifier
    pub digital_asset_id: [u8; 32],
    /// Blockchain where transfer originated
    pub origin_network: u64,
    /// Target blockchain for transfer
    pub target_network: u64,
    /// User who initiated the transfer
    pub transfer_initiator: Pubkey,
    /// Final destination address
    pub destination_address: String,
    /// Asset metadata location
    pub metadata_location: String,
    /// Transfer fee amount
    pub transfer_fee: u64,
    /// When transfer was started
    pub transfer_time: i64,
    /// Transfer tracking number
    pub transfer_number: u64,
}

/// Event emitted when a digital asset is destroyed during transfer
#[event]
pub struct DigitalAssetDestroyed {
    /// Token mint address
    pub token_mint: Pubkey,
    /// Digital asset identifier
    pub digital_asset_id: [u8; 32],
    /// User who owned the asset
    pub previous_owner: Pubkey,
    /// When asset was destroyed
    pub destruction_time: i64,
    /// Purpose of destruction
    pub destruction_purpose: String,
}
