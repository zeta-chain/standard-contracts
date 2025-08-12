use anchor_lang::prelude::*;

#[account]
pub struct ReplayMarker {
    pub token_id: [u8; 32],
    pub nonce: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl ReplayMarker {
    pub const SEED: &'static [u8] = b"replay";
    pub const LEN: usize = 32 + 8 + 8 + 1;
}
