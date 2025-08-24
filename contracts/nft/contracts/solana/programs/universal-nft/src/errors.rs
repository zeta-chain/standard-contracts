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
}
