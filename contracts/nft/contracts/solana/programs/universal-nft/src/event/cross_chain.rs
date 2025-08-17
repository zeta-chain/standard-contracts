use anchor_lang::prelude::*;

/// Cross-chain transfer events
#[event]
pub struct CrossChainTransferInitiated {
    /// Universal token ID being transferred
    pub token_id: [u8; 32],
    /// Source chain (Solana)
    pub source_chain: u64,
    /// Destination chain ID
    pub destination_chain: u64,
    /// Sender on Solana
    pub sender: Pubkey,
    /// Recipient address on destination chain
    pub recipient: String,
    /// NFT metadata URI
    pub metadata_uri: String,
    /// Gas limit for the operation
    pub gas_limit: u64,
    /// Timestamp of transfer initiation
    pub timestamp: i64,
    /// Nonce for replay protection
    pub nonce: u64,
}

#[event]
pub struct CrossChainTransferCompleted {
    /// Universal token ID that was transferred
    pub token_id: [u8; 32],
    /// Source chain ID
    pub source_chain: u64,
    /// Destination chain (Solana)
    pub destination_chain: u64,
    /// Recipient on Solana
    pub recipient: Pubkey,
    /// New mint account created on Solana
    pub new_mint: Pubkey,
    /// Original metadata URI
    pub metadata_uri: String,
    /// Timestamp of transfer completion
    pub timestamp: i64,
    /// Whether this NFT was originally from Solana
    pub is_original_solana_nft: bool,
}

#[event]
pub struct CrossChainTransferFailed {
    /// Universal token ID that failed to transfer
    pub token_id: [u8; 32],
    /// Source chain ID
    pub source_chain: u64,
    /// Destination chain ID
    pub destination_chain: u64,
    /// Error message
    pub error_message: String,
    /// Timestamp of failure
    pub timestamp: i64,
    /// Nonce of the failed operation
    pub nonce: u64,
}

#[event]
pub struct CrossChainTransferReverted {
    /// Original sender who initiated the transfer
    pub sender: Pubkey,
    /// Amount of SOL refunded (if any)
    pub amount: u64,
    /// Reason for the revert
    pub revert_reason: String,
    /// Timestamp of revert
    pub timestamp: i64,
    /// Whether an NFT was restored during revert
    pub nft_restored: bool,
}
