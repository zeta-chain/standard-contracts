use anchor_lang::prelude::*;
use solana_program::program_error::ProgramError;

/// Verify a ZetaChain message signature from the gateway set.
///
/// For now we provide a safe default: unless the `insecure-placeholder` feature
/// is explicitly enabled, verification fails closed so this cannot be used
/// in production accidentally.
#[cfg(feature = "insecure-placeholder")]
pub fn verify_zeta_signature(_message: &[u8], _signature: &[u8]) -> Result<()> {
    msg!("Verifying signature... [placeholder]");
    Ok(())
}

#[cfg(not(feature = "insecure-placeholder"))]
pub fn verify_zeta_signature(_message: &[u8], _signature: &[u8]) -> Result<()> {
    // TODO: replace with real verification using on-chain sysvars or ed25519/secp256k1 programs.
    Err(ProgramError::MissingRequiredSignature.into())
}
