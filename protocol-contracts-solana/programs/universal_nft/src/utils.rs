use anchor_lang::prelude::*;

pub fn derive_nft_origin_pda(token_id: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"nft_origin", token_id], &crate::ID)
}
