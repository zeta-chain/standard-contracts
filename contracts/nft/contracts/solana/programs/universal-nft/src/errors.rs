use anchor_lang::prelude::*;

#[error_code]
pub enum UniversalNftError {
    #[msg("Unauthorized gateway")]
    UnauthorizedGateway,
    #[msg("Invalid message format")]
    InvalidMessage,
    #[msg("Invalid recipient address")]
    InvalidRecipient,
    #[msg("Insufficient compute budget")]
    InsufficientComputeBudget,
    #[msg("Token account creation failed")]
    TokenAccountCreationFailed,
    #[msg("Invalid collection authority")]
    InvalidCollectionAuthority,
    #[msg("Collection not found")]
    CollectionNotFound,
    #[msg("NFT mint failed")]
    NftMintFailed,
    #[msg("Cross-chain transfer failed")]
    CrossChainTransferFailed,
    #[msg("Invalid chain ID")]
    InvalidChainId,
    #[msg("Contract address not set")]
    ContractAddressNotSet,
    #[msg("Rent exemption failed")]
    RentExemptionFailed,
    #[msg("Invalid name length")]
    InvalidNameLength,
    #[msg("Invalid symbol length")]
    InvalidSymbolLength,
    #[msg("Invalid URI length")]
    InvalidUriLength,
    #[msg("Invalid token ID")]
    InvalidTokenId,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Invalid destination chain")]
    InvalidDestinationChain,
    #[msg("Unsupported chain")]
    UnsupportedChain,
    #[msg("Invalid recipient address")]
    InvalidRecipientAddress,
    #[msg("Insufficient gas amount")]
    InsufficientGasAmount,
}
