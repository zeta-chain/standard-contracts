use anchor_lang::prelude::*;
use crate::util::constants::NFT_ORIGIN_SPACE;

/// NFT origin tracking account
/// Maps universal token IDs to their original Solana mint accounts
#[account]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftOrigin {
    /// Universal token ID used across all chains
    pub token_id: [u8; 32],
    /// Original Solana mint account (for Solana-originated NFTs)
    pub original_mint: Pubkey,
    /// Original metadata account
    pub original_metadata: Pubkey,
    /// Original metadata URI
    pub original_uri: String,
    /// When this NFT was first minted on Solana
    pub created_at: i64,
    /// Current timestamp of last update
    pub updated_at: i64,
    /// Whether this NFT is currently on Solana
    pub is_on_solana: bool,
    /// Bump seed for PDA
    pub bump: u8,
}

impl NftOrigin {
    pub const LEN: usize = NFT_ORIGIN_SPACE;
    
    /// Create a new NFT origin tracking entry
    pub fn new(
        token_id: [u8; 32],
        original_mint: Pubkey,
        original_metadata: Pubkey,
        original_uri: String,
        created_at: i64,
        bump: u8,
    ) -> Self {
        Self {
            token_id,
            original_mint,
            original_metadata,
            original_uri,
            created_at,
            updated_at: created_at,
            is_on_solana: true,
            bump,
        }
    }
    
    /// Mark NFT as transferred off Solana
    pub fn mark_transferred_off_solana(&mut self, timestamp: i64) {
        self.is_on_solana = false;
        self.updated_at = timestamp;
    }
    
    /// Mark NFT as returned to Solana
    pub fn mark_to_solana(&mut self, timestamp: i64) {
        self.is_on_solana = true;
        self.updated_at = timestamp;
    }
}
