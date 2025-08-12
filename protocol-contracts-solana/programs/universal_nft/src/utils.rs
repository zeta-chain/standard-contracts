use anchor_lang::prelude::*;

pub fn derive_nft_origin_pda(token_id: &[u8]) -> (Pubkey, u8) {
    // Seed order: [token_id_bytes, "nft_origin"] per spec
    Pubkey::find_program_address(&[token_id, b"nft_origin"], &crate::ID)
}

pub fn derive_replay_marker_pda(token_id: &[u8], nonce: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"replay", token_id, &nonce.to_le_bytes()], &crate::ID)
}

pub fn derive_metadata_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"metadata", &mpl_token_metadata::ID.as_ref(), mint.as_ref()],
        &mpl_token_metadata::ID,
    )
}

pub fn derive_master_edition_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            &mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            b"edition",
        ],
        &mpl_token_metadata::ID,
    )
}
