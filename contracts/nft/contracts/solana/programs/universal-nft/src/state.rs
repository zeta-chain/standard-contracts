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

/// Token reservation account for pre-allocated unique identifiers
#[account]
#[derive(InitSpace)]
pub struct TokenReservation {
    /// The mint address this voucher is for
    pub mint_address: Pubkey,
    /// The authority that created this voucher
    pub creator: Pubkey,
    /// The reserved token identifier number
    pub reserved_id: u64,
    /// The blockchain slot when voucher was created
    pub block_slot: u64,
    /// The computed unique token hash
    pub token_hash: [u8; 32],
    /// Whether this voucher has been used
    pub is_consumed: bool,
    /// Timestamp when voucher was created
    pub creation_time: i64,
    /// PDA bump seed
    pub bump_seed: u8,
}

/// Universal NFT origin tracking account
#[account]
#[derive(InitSpace)]
pub struct UniversalNftOrigin {
    /// Unique identifier for the Universal NFT
    pub nft_id: [u8; 32],
    /// Original mint address
    pub original_mint: Pubkey,
    /// Original metadata address
    pub original_metadata: Pubkey,
    /// Original URI
    #[max_len(200)]
    pub original_uri: String,
    /// Whether NFT is currently on Solana
    pub is_on_solana: bool,
    /// Timestamp when NFT was created
    pub created_at: i64,
    /// Timestamp when NFT was transferred off Solana
    pub transferred_at: Option<i64>,
    /// PDA bump seed
    pub bump_seed: u8,
}

impl UniversalNftOrigin {
    /// Mark the NFT as transferred off Solana
    pub fn mark_transferred_off_solana(&mut self, timestamp: i64) {
        self.is_on_solana = false;
        self.transferred_at = Some(timestamp);
    }
}
