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
    pub const SEED: &'static [u8] = b"nft_origin";
    pub const MAX_URI_LEN: usize = 200; // adjust as needed
    // Account discriminator (8) is added by Anchor at allocation site
    pub const LEN: usize = 2  // origin_chain
        + 32                 // origin_token_id
        + 32                 // origin_mint
        + 4 + Self::MAX_URI_LEN // metadata_uri (borsh string prefix + bytes)
        + 8                  // created_at
        + 1;                 // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrossChainNftPayload {
    pub version: u8,
    pub token_id: [u8; 32],
    pub origin_chain_id: u16,
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub recipient: Pubkey,
    pub nonce: u64,
}
