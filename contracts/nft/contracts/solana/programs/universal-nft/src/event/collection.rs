use anchor_lang::prelude::*;

/// Collection management events
#[event]
pub struct CollectionCreated {
    /// Collection account public key
    pub collection: Pubkey,
    /// Collection name
    pub name: String,
    /// Collection metadata URI
    pub uri: String,
    /// Collection authority
    pub authority: Pubkey,
    /// Maximum number of NFTs in collection
    pub max_supply: Option<u64>,
    /// Timestamp when created
    pub timestamp: i64,
}

#[event]
pub struct CollectionUpdated {
    /// Collection account public key
    pub collection: Pubkey,
    /// New metadata URI (if changed)
    pub new_uri: Option<String>,
    /// New authority (if changed)
    pub new_authority: Option<Pubkey>,
    /// Updated by
    pub updated_by: Pubkey,
    /// Timestamp of update
    pub timestamp: i64,
}

#[event]
pub struct NftAddedToCollection {
    /// NFT mint account
    pub mint: Pubkey,
    /// Universal token ID
    pub token_id: [u8; 32],
    /// Collection it was added to
    pub collection: Pubkey,
    /// Who added it (must be collection authority)
    pub added_by: Pubkey,
    /// Timestamp
    pub timestamp: i64,
}
