use anchor_lang::prelude::*;

/// Token metadata structure compatible with Metaplex
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenMetadata {
    /// NFT name
    pub name: String,
    /// NFT symbol
    pub symbol: String,
    /// Metadata URI
    pub uri: String,
    /// Seller fee basis points (royalties)
    pub seller_fee_basis_points: u16,
    /// Creator information
    pub creators: Option<Vec<Creator>>,
    /// Collection information
    pub collection: Option<Collection>,
    /// Uses information
    pub uses: Option<Uses>,
}

/// Creator information for royalties
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator {
    /// Creator's public key
    pub address: Pubkey,
    /// Whether the creator has verified the NFT
    pub verified: bool,
    /// Percentage share (0-100)
    pub share: u8,
}

/// Collection information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Collection {
    /// Collection mint account
    pub key: Pubkey,
    /// Whether the collection has been verified
    pub verified: bool,
}

/// Uses information (for utility NFTs)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Uses {
    /// Use method
    pub use_method: UseMethod,
    /// Remaining uses
    pub remaining: u64,
    /// Total uses allowed
    pub total: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}
