use anchor_lang::prelude::*;

#[account]
pub struct ReplayMarker {
    pub token_id: [u8; 32],
    pub nonce: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl ReplayMarker {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1; // discriminator + [u8; 32] + u64 + i64 + u8
    pub const SEED: &[u8] = b"replay";
}
