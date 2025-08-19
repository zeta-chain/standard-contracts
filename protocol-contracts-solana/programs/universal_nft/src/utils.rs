use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use mpl_token_metadata::instruction as mpl_ix;
use mpl_token_metadata::state::DataV2;

pub fn derive_nft_origin_pda(token_id: &[u8]) -> (Pubkey, u8) {
    // Seed order: [token_id_bytes, "nft_origin"] per spec
    Pubkey::find_program_address(&[token_id, b"nft_origin"], &crate::ID)
}

pub fn derive_replay_marker_pda(token_id: &[u8], nonce: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"replay", token_id, &nonce.to_le_bytes()], &crate::ID)
}

pub fn derive_metadata_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[
        b"metadata",
        mpl_token_metadata::ID.as_ref(),
        mint.as_ref(),
    ], &mpl_token_metadata::ID)
}

pub fn derive_master_edition_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[
        b"metadata",
        mpl_token_metadata::ID.as_ref(),
        mint.as_ref(),
        b"edition",
    ], &mpl_token_metadata::ID)
}

pub fn cpi_create_metadata_v3(
    payer: &AccountInfo,
    mint: &AccountInfo,
    mint_authority: &AccountInfo,
    update_authority: &AccountInfo,
    metadata: &AccountInfo,
    system_program: &AccountInfo,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let data = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let ix = mpl_ix::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        *metadata.key,
        *mint.key,
        *mint_authority.key,
        *payer.key,
        *update_authority.key,
        data.name.clone(),
        data.symbol.clone(),
        data.uri.clone(),
        None,
        0,
        true,
        true,
        None,
        None,
        None,
    );

    invoke_signed(&ix, &[metadata.clone(), mint.clone(), mint_authority.clone(), payer.clone(), update_authority.clone(), system_program.clone()], &[])?;

    Ok(())
}

pub fn cpi_create_master_edition_v3(
    payer: &AccountInfo,
    mint: &AccountInfo,
    mint_authority: &AccountInfo,
    update_authority: &AccountInfo,
    metadata: &AccountInfo,
    master_edition: &AccountInfo,
) -> Result<()> {
    let ix = mpl_ix::create_master_edition_v3(
        mpl_token_metadata::ID,
        *master_edition.key,
        *mint.key,
        *update_authority.key,
        *mint_authority.key,
        *metadata.key,
        *payer.key,
        Some(0),
    );

    invoke_signed(&ix, &[master_edition.clone(), mint.clone(), update_authority.clone(), mint_authority.clone(), metadata.clone(), payer.clone()], &[])?;

    Ok(())
}
