use anchor_lang::prelude::*;

#[error_code]
pub enum UniversalNftError {
    // ========== ACCOUNT VALIDATION ERRORS ==========
    #[msg("Invalid program authority")]
    InvalidAuthority,
    
    #[msg("Account not properly initialized")]
    AccountNotInitialized,
    
    #[msg("Account already initialized")]
    AccountAlreadyInitialized,
    
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    
    #[msg("Account discriminator mismatch")]
    AccountDiscriminatorMismatch,

    // ========== INPUT VALIDATION ERRORS ==========
    #[msg("Token ID too long")]
    TokenIdTooLong,
    
    #[msg("Token ID too short")]
    TokenIdTooShort,
    
    #[msg("Invalid token ID format")]
    InvalidTokenId,
    
    #[msg("URI too long")]
    UriTooLong,
    
    #[msg("Name too long")]
    NameTooLong,
    
    #[msg("Symbol too long")]
    SymbolTooLong,
    
    #[msg("Invalid URI format")]
    InvalidUri,

    // ========== NFT OPERATION ERRORS ==========
    #[msg("NFT does not exist")]
    NftDoesNotExist,
    
    #[msg("NFT already exists")]
    NftAlreadyExists,
    
    #[msg("Program is paused")]
    ProgramPaused,
    
    #[msg("Invalid token name")]
    InvalidTokenName,
    
    #[msg("Invalid token URI")]
    InvalidTokenUri,
    
    
    #[msg("Not the NFT owner")]
    NotNftOwner,
    
    #[msg("NFT is frozen")]
    NftFrozen,
    
    #[msg("NFT already burned")]
    NftAlreadyBurned,
    
    #[msg("Cannot burn NFT")]
    CannotBurnNft,

    // ========== CROSS-CHAIN ERRORS ==========
    #[msg("Unsupported destination chain")]
    UnsupportedDestinationChain,
    
    #[msg("Invalid chain ID")]
    InvalidChainId,
    
    #[msg("Cross-chain message expired")]
    MessageExpired,
    
    #[msg("Invalid cross-chain message")]
    InvalidCrossChainMessage,
    
    #[msg("Cross-chain operation failed")]
    CrossChainOperationFailed,
    
    #[msg("Invalid recipient address")]
    InvalidRecipientAddress,
    
    #[msg("Gateway not configured")]
    GatewayNotConfigured,

    // ========== METAPLEX ERRORS ==========
    #[msg("Metadata creation failed")]
    MetadataCreationFailed,
    
    #[msg("Master edition creation failed")]
    MasterEditionCreationFailed,
    
    #[msg("Invalid program provided")]
    InvalidProgram,
    
    #[msg("Invalid gateway program")]
    InvalidGatewayProgram,
    
    #[msg("Invalid mint account")]
    InvalidMint,
    
    #[msg("Invalid metadata account")]
    InvalidMetadata,
    
    #[msg("Invalid token amount - must be 1 for NFTs")]
    InvalidTokenAmount,
    
    #[msg("Cross-chain transfer failed")]
    CrossChainTransferFailed,
    
    #[msg("Invalid metadata account")]
    InvalidMetadataAccount,
    
    #[msg("Metadata update failed")]
    MetadataUpdateFailed,

    // ========== COLLECTION ERRORS ==========
    #[msg("Collection does not exist")]
    CollectionDoesNotExist,
    
    #[msg("Collection already exists")]
    CollectionAlreadyExists,
    
    #[msg("Not collection authority")]
    NotCollectionAuthority,
    
    #[msg("Collection is full")]
    CollectionFull,
    
    #[msg("Invalid collection configuration")]
    InvalidCollectionConfig,

    // ========== ARITHMETIC ERRORS ==========
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
    
    #[msg("Division by zero")]
    DivisionByZero,

    // ========== TIMING ERRORS ==========
    #[msg("Operation too soon")]
    OperationTooSoon,
    
    #[msg("Operation timeout")]
    OperationTimeout,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,

    // ========== SECURITY ERRORS ==========
    #[msg("Unauthorized operation")]
    Unauthorized,
    
    #[msg("Replay attack detected")]
    ReplayAttack,
    
    #[msg("Invalid signature")]
    InvalidSignature,
    
    #[msg("Nonce mismatch")]
    NonceMismatch,
    
    #[msg("Operation rate limit exceeded")]
    RateLimitExceeded,

    // ========== CONFIGURATION ERRORS ==========
    #[msg("Invalid gas limit")]
    InvalidGasLimit,
    
    #[msg("Configuration not found")]
    ConfigurationNotFound,
    
    #[msg("Invalid configuration")]
    InvalidConfiguration,

    // ========== GENERAL ERRORS ==========
    #[msg("Feature not implemented")]
    NotImplemented,
    
    #[msg("Internal program error")]
    InternalError,
    
    #[msg("Invalid program state")]
    InvalidProgramState,
    
    #[msg("Operation not allowed")]
    OperationNotAllowed,
    
    #[msg("Serialization error")]
    SerializationError,
    
    #[msg("Invalid data format")]
    InvalidDataFormat,
    
    #[msg("Invalid caller - only gateway can call this function")]
    InvalidCaller,

    // ========== ORIGIN/STATE ERRORS ==========
    #[msg("NFT not currently on Solana")]
    NftNotOnSolana,

    #[msg("Invalid token supply for NFT")]
    InvalidTokenSupply,
}
