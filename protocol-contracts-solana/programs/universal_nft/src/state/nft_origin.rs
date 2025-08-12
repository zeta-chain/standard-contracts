use anchor_lang::prelude::*;

#[account]
pub struct NftOrigin {
    pub origin_chain: u16,
    pub origin_token_id: Vec<u8>,
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub created_at: i64,
    pub bump: u8,
}

impl NftOrigin {
    pub const SEED: &'static [u8] = b"nft_origin";
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrossChainNftPayload {
    pub version: u8,
    pub token_id: Vec<u8>,
    pub origin_chain_id: u16,
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub recipient: Pubkey,
    pub nonce: u64,
}
