use anchor_lang::prelude::*;

/// Program configuration account
/// Stores global settings for the Universal NFT program
#[account]
#[derive(Debug)]
pub struct UniversalNftConfig {
    /// Program authority who can update configurations
    pub authority: Pubkey,
    /// ZetaChain gateway program for cross-chain operations
    pub gateway_program: Pubkey,
    /// Expected ZetaChain gateway PDA (pin the exact authority account)
    pub gateway_pda: Pubkey,
    /// Current nonce for replay protection
    pub nonce: u64,
    /// Next token id counter for new mints
    pub next_token_id: u64,
    /// Whether the program is paused
    pub is_paused: bool,
    /// Timestamp when created
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl UniversalNftConfig {
    /// Payload length (excluding the 8-byte Anchor discriminator)
    pub const LEN: usize = 32 // authority
        + 32                  // gateway_program
        + 32                  // gateway_pda
        + 8                   // nonce
        + 8                   // next_token_id
        + 1                   // is_paused
        + 8                   // created_at
        + 1;                  // bump
    /// Total account space including discriminator
    pub const SPACE: usize = 8 + Self::LEN;
}
