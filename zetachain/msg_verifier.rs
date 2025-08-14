//! ZetaChain message signature verification utilities.
//!
//! This module provides a placeholder verifier that is **feature-gated**
//! so accidental use in production cannot silently bypass checks.
//
//! Usage:
//! - Enable the "insecure-placeholder" feature ONLY in local dev/tests to
//!   keep behavior compatible while a real verifier is implemented.
//! - Without the feature, calls will return `MissingRequiredSignature`.
//
//! Replace this with a real verifier that checks for a preceding
//! ed25519/secp256k1 verification instruction (preferred on Solana).

use solana_program::{msg, program_error::ProgramError};

/// Verify the ZetaChain message signature.
///
/// When the `insecure-placeholder` feature is enabled, this function
/// logs a placeholder message and returns `Ok(())` for convenience in
/// local development. Otherwise, it returns
/// `ProgramError::MissingRequiredSignature` to prevent accidental bypasses.
#[cfg(feature = "insecure-placeholder")]
pub fn verify_zeta_signature(_message: &[u8], _signature: &[u8]) -> Result<(), ProgramError> {
    msg!("Verifying signature... [placeholder]");
    Ok(())
}

#[cfg(not(feature = "insecure-placeholder"))]
pub fn verify_zeta_signature(_message: &[u8], _signature: &[u8]) -> Result<(), ProgramError> {
    // TODO: Implement real verification:
    //  - Prefer using Solana's built-in ed25519/secp256k1 verification syscalls
    //    by asserting a prior instruction matches the expected signer and message.
    //  - Or perform domain-specific checks required by ZetaChain's gateway.
    Err(ProgramError::MissingRequiredSignature)
}
