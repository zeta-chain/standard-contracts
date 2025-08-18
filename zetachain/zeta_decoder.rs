use anchor_lang::prelude::*;
use solana_program::program_error::ProgramError;
use anchor_lang::solana_program::pubkey::Pubkey;

/// Minimal NFT message structure expected from ZetaChain.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct NFTMessage {
    pub uri: String,
    pub title: String,
    pub symbol: String,
    pub recipient: Pubkey,
    pub nonce: u64,
}

/// Decode a ZetaChain payload into an `NFTMessage`.
///
/// By default (without feature flags), this function fails closed so we don't
/// accept arbitrary payloads in production. Enable `insecure-placeholder` for
/// local testing to return a dummy message.
#[cfg(feature = "insecure-placeholder")]
pub fn decode_zeta_payload(payload: &[u8]) -> Result<NFTMessage> {
    msg!("Decoding payload... [placeholder]");
    if payload.is_empty() {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    Ok(NFTMessage {
        uri: "ipfs://dummy_uri".to_string(),
        title: "ZetaNFT".to_string(),
        symbol: "ZETA".to_string(),
        recipient: Pubkey::default(),
        nonce: 0,
    })
}

#[cfg(not(feature = "insecure-placeholder"))]
pub fn decode_zeta_payload(payload: &[u8]) -> Result<NFTMessage> {
    // Until a real decoder is implemented, fail closed.
    if payload.is_empty() {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    Err(ProgramError::InvalidInstructionData.into())
}
