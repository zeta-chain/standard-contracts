use anchor_lang::prelude::*;

/// Errors that can occur during execution.
#[error_code]
pub enum Errors {
    #[msg("Provided gateway program ID cannot be the default pubkey")]
    GatewayProgramDefault,

    #[msg("Provided gateway program account does not match the expected ID")]
    GatewayProgramMismatch,

    #[msg("Gateway program account is not executable")]
    GatewayProgramNotExecutable,

    #[msg("Gateway PDA is not owned by the gateway program")]
    GatewayPdaOwnershipInvalid,

    // Token reservation specific errors
    #[msg("Token reservation has already been consumed")]
    ReservationAlreadyUsed,

    #[msg("Invalid token reservation data")]
    InvalidReservationData,

    #[msg("Mathematical operation overflow occurred")]
    MathOverflow,

    #[msg("Program is currently paused")]
    ProgramPaused,

    // Digital asset creation errors
    #[msg("Digital asset voucher has already been consumed")]
    VoucherAlreadyUsed,

    #[msg("Invalid digital asset voucher data")]
    InvalidVoucherData,

    #[msg("Digital asset name is too long")]
    NameTooLong,

    #[msg("Digital asset symbol is too long")]
    SymbolTooLong,

    #[msg("Digital asset URI is too long")]
    UriTooLong,

    #[msg("Insufficient rent provided for account creation")]
    InsufficientRent,

    #[msg("Invalid mint account")]
    InvalidMint,

    #[msg("Invalid token supply")]
    InvalidTokenSupply,

    #[msg("Invalid token amount")]
    InvalidTokenAmount,

    #[msg("Invalid recipient address")]
    InvalidRecipientAddress,

    #[msg("Invalid program")]
    InvalidProgram,

    #[msg("Invalid data format")]
    InvalidDataFormat,

    #[msg("Invalid account owner")]
    InvalidAccountOwner,

    #[msg("Origin conflict detected")]
    OriginConflict,

    #[msg("Operation not allowed")]
    OperationNotAllowed,

    #[msg("Invalid bridge program")]
    InvalidBridgeProgram,

    #[msg("Invalid metadata account")]
    InvalidMetadata,

    #[msg("Asset not on Solana")]
    AssetNotOnSolana,

    #[msg("Invalid caller")]
    InvalidCaller,

    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,

    #[msg("Unauthorized access attempt")]
    UnauthorizedAccess,

    #[msg("Invalid parameter provided")]
    InvalidParameter,
}
