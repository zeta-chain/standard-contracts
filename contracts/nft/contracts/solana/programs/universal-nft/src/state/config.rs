use anchor_lang::prelude::*;
use crate::util::constants::UNIVERSAL_NFT_CONFIG_SPACE;

/// Program configuration account
/// Stores global settings for the Universal NFT program
#[account]
#[derive(Debug)]
pub struct UniversalNftConfig {
    /// Program authority who can update configurations
    pub authority: Pubkey,
    /// ZetaChain gateway program for cross-chain operations
    pub gateway_program: Pubkey,
    /// Current nonce for replay protection
    pub nonce: u64,
    /// Whether the program is paused
    pub is_paused: bool,
    /// Timestamp when created
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl UniversalNftConfig {
    pub const LEN: usize = UNIVERSAL_NFT_CONFIG_SPACE;
}
