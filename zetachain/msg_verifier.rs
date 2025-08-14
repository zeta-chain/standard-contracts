
//! ZetaChain message signature verification utilities.
//!
//! NOTE: In production you should implement real verification or
//! leverage Solana's built-in ed25519/secp256k1 instruction checks.
//! This module provides a *feature-gated* placeholder so that
//! accidental "always true" logic cannot slip into release builds.

use anchor_lang::prelude::*;

/// Verify a ZetaChain message signature.
///
/// In normal (secure) builds, this returns an error until a real
/// verification is implemented. When the crate is compiled with the
/// `insecure-placeholder` feature, it will always succeed and emit
/// a log. This makes tests/dev flows convenient while preventing
/// silent bypasses in production.
#[cfg(feature = "insecure-placeholder")]
pub fn verify_zeta_signature(_message: &[u8], _signature: &[u8]) -> Result<()> {
    msg!("Verifying signature... [placeholder - insecure build feature enabled]");
    Ok(())
}

/// Secure default: fail closed until verification is implemented.
#[cfg(not(feature = "insecure-placeholder"))]
pub fn verify_zeta_signature(_message: &[u8], _signature: &[u8]) -> Result<()> {
    // TODO: Replace with real verification logic.
    Err(ProgramError::MissingRequiredSignature)
}
