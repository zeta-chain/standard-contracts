use anchor_lang::prelude::*;

/// Origin tracking events
#[event]
pub struct OriginTracked {
    /// Universal token ID
    pub token_id: [u8; 32],
    /// Original mint account (for Solana-originated NFTs)
    pub original_mint: Pubkey,
    /// Original metadata account
    pub original_metadata: Pubkey,
    /// Original metadata URI
    pub original_uri: String,
    /// When the origin was first tracked
    pub created_at: i64,
    /// Current timestamp
    pub timestamp: i64,
}

#[event]
pub struct OriginRestored {
    /// Universal token ID
    pub token_id: [u8; 32],
    /// Original mint that was restored
    pub original_mint: Pubkey,
    /// New mint account created for restoration
    pub new_mint: Pubkey,
    /// Restored to owner
    pub owner: Pubkey,
    /// Timestamp
    pub timestamp: i64,
}
