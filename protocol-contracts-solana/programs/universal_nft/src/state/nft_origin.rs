use anchor_lang::prelude::*;

#[account]
pub struct NftOrigin {
    pub origin_chain: u16,
    pub origin_token_id: [u8; 32],
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub created_at: i64,
    pub bump: u8,
}

impl NftOrigin {
    pub const LEN: usize = 8 + 2 + 32 + 32 + 4 + 200 + 8 + 1; // discriminator + u16 + [u8; 32] + pubkey + string_len + max_uri + i64 + u8
    pub const MAX_URI_LEN: usize = 200;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CrossChainNftPayload {
    pub token_id: [u8; 32],
    pub origin_chain_id: u16,
    pub origin_mint: Pubkey,
    pub recipient: Pubkey,
    pub metadata_uri: String,
    pub nonce: u64,
}
