use anchor_lang::prelude::*;

#[error_code]
pub enum UniversalNftError {
    #[msg("Program is not initialized")]
    ProgramNotInitialized,
    
    #[msg("Program is already initialized")]
    ProgramAlreadyInitialized,
    
    #[msg("Invalid authority")]
    InvalidAuthority,
    
    #[msg("Invalid gateway program ID")]
    InvalidGatewayProgramId,
    
    #[msg("Invalid TSS address")]
    InvalidTssAddress,
    
    #[msg("Invalid signature")]
    InvalidSignature,
    
    #[msg("Invalid signature recovery")]
    InvalidSignatureRecovery,
    
    #[msg("Message already processed")]
    MessageAlreadyProcessed,
    
    #[msg("Invalid nonce")]
    InvalidNonce,
    
    #[msg("Invalid chain ID")]
    InvalidChainId,
    
    #[msg("Invalid token ID")]
    InvalidTokenId,
    
    #[msg("NFT is locked for cross-chain transfer")]
    NftLockedForCrossChain,
    
    #[msg("NFT not found")]
    NftNotFound,
    
    #[msg("Invalid metadata")]
    InvalidMetadata,
    
    #[msg("Metadata too long")]
    MetadataTooLong,
    
    #[msg("Invalid destination address")]
    InvalidDestinationAddress,
    
    #[msg("Invalid message format")]
    InvalidMessageFormat,
    
    #[msg("Cross-chain history limit exceeded")]
    CrossChainHistoryLimitExceeded,
    
    #[msg("Insufficient funds for cross-chain transfer")]
    InsufficientFundsForCrossChain,
    
    #[msg("Invalid mint authority")]
    InvalidMintAuthority,
    
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    
    #[msg("Token account not owned by program")]
    TokenAccountNotOwnedByProgram,
    
    #[msg("Mathematical overflow occurred")]
    MathematicalOverflow,
    
    #[msg("Invalid account data")]
    InvalidAccountData,
    
    #[msg("Unauthorized cross-chain operation")]
    UnauthorizedCrossChainOperation,
    
    #[msg("Gateway call failed")]
    GatewayCallFailed,
    
    #[msg("Invalid revert context")]
    InvalidRevertContext,
    
    #[msg("Revert operation failed")]
    RevertOperationFailed,
    
    #[msg("Collection not initialized")]
    CollectionNotInitialized,
    
    #[msg("Invalid collection mint")]
    InvalidCollectionMint,
    
    #[msg("Creator verification failed")]
    CreatorVerificationFailed,
    
    #[msg("Attributes limit exceeded")]
    AttributesLimitExceeded,
    
    #[msg("Invalid attribute data")]
    InvalidAttributeData,
}