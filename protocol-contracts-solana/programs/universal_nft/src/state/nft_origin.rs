use anchor_lang::prelude::*;

#[account]
pub struct NftOrigin {
    pub origin_chain: u64,
    pub origin_token_id: [u8; 32],
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub created_at: i64,
    pub bump: u8,
}

impl NftOrigin {
    pub const SEED: &'static [u8] = b"nft_origin";
    pub const MAX_URI_LEN: usize = 200;
    // Space calculation (excludes 8-byte discriminator):
    pub const LEN: usize = 8           // origin_chain (u64)
        + 32                           // origin_token_id (fixed array)
        + 32                           // origin_mint (Pubkey)
        + 4 + Self::MAX_URI_LEN       // metadata_uri (length prefix + data)
        + 8                            // created_at (i64)
        + 1;                           // bump (u8)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainNftPayload {
    pub token_id: [u8; 32],
    pub origin_chain: u64,
    pub origin_mint: Pubkey,
    pub recipient: Pubkey,
    pub metadata_uri: String,
    pub nonce: u64,
}
