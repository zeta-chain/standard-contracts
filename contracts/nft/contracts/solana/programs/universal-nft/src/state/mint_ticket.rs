use anchor_lang::prelude::*;

#[account]
pub struct MintTicket {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub reserved_id: u64,
    pub slot: u64,
    pub token_id: [u8; 32],
    pub used: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl MintTicket {
    /// Payload length in bytes (excludes the 8-byte Anchor discriminator).
    pub const LEN: usize = 32  // mint
        + 32                  // authority
        + 8                   // reserved_id
        + 8                   // slot
        + 32                  // token_id
        + 1                   // used
        + 8                   // created_at
        + 1;                  // bump
    /// Total on-chain space in bytes (discriminator + payload). Use this for allocations.
    pub const SPACE: usize = 8 + Self::LEN;
}
