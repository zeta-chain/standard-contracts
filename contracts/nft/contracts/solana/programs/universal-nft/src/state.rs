use anchor_lang::prelude::*;

/// Configuration account for the Universal NFT program
#[account]
#[derive(InitSpace)]
pub struct UniversalNftConfig {
    /// Optional authority that can update config settings
    pub admin: Option<Pubkey>,
    /// Program ID of the Zeta Gateway program for cross-chain messaging
    pub zeta_gateway_program_id: Pubkey,
    /// PDA account owned by Gateway program that verifies cross-chain messages
    pub zeta_gateway_verifier: Pubkey,
    /// Incrementing nonce for unique message IDs
    pub message_sequence: u64,
    /// Next available token ID for minting NFTs, incremented after each mint
    pub next_nft_id: u64,
    /// Flag to pause/unpause program functionality
    pub paused: bool,
    /// Unix timestamp when config was created
    pub initialized_timestamp: i64,
    /// Bump seed used to derive config PDA
    pub pda_bump: u8,
}
