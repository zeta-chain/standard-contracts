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
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 32 + 1 + 8 + 1;
}
