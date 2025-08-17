use anchor_lang::prelude::*;

/// NFT lifecycle events
#[event]
pub struct NftMinted {
    /// The mint account of the newly created NFT
    pub mint: Pubkey,
    /// Universal token ID for cross-chain compatibility
    pub token_id: [u8; 32],
    /// NFT owner/recipient
    pub owner: Pubkey,
    /// Metadata URI
    pub uri: String,
    /// Collection this NFT belongs to (if any)
    //pub collection: Option<Pubkey>,
    /// Timestamp when minted
    pub timestamp: i64,
}

#[event]
pub struct NftBurned {
    /// The mint account of the burned NFT
    pub mint: Pubkey,
    /// Universal token ID
    pub token_id: [u8; 32],
    /// Previous owner of the NFT
    pub owner: Pubkey,
    /// Timestamp when burned
    pub timestamp: i64,
    /// Reason for burning (cross-chain transfer, etc.)
    pub reason: String,
}
