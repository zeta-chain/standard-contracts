
//! ZetaChain payload decoder (temporary scaffold).
//!
//! Adds basic input validation so empty payloads do not silently pass.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;

// If this struct is defined elsewhere in your crate, remove this duplicate
// and import it instead. It is included here to make the file self-contained.
#[derive(Clone, Debug)]
pub struct NFTMessage {
    pub uri: String,
    pub title: String,
    pub symbol: String,
    pub recipient: Pubkey,
    pub nonce: u64,
}

/// Decode a ZetaChain payload into an `NFTMessage`.
///
/// This is a placeholder decoder that returns dummy values for now,
/// but it performs a minimal guard against empty payloads to avoid
/// masking integration errors.
pub fn decode_zeta_payload(payload: &[u8]) -> Result<NFTMessage> {
    msg!("Decoding payload... [placeholder]");

    // Minimal guard: avoid accepting empty payloads.
    if payload.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // TODO: Parse the actual bytes in `payload` into your message format.
    Ok(NFTMessage {
        uri: "ipfs://dummy_uri".to_string(),
        title: "ZetaNFT".to_string(),
        symbol: "ZETA".to_string(),
        recipient: Pubkey::default(),
        nonce: 0,
    })
}
