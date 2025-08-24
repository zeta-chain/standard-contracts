use anchor_lang::prelude::*;

/// Event emitted when an NFT is recovered from cross-chain bridge
#[event]
pub struct NftRecovered {
    /// The mint address of the recovered NFT
    pub mint: Pubkey,
    /// The unique identifier of the NFT
    pub nft_id: [u8; 32],
    /// The owner of the recovered NFT (program PDA)
    pub owner: Pubkey,
    /// The original URI of the NFT
    pub uri: String,
    /// The timestamp when the NFT was recovered
    pub timestamp: i64,
    /// The origin chain where the NFT came from
    pub origin_chain: u64,
}
