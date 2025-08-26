use anchor_lang::prelude::*;
use mpl_token_metadata::types::DataV2;
use anchor_spl::metadata::{create_master_edition_v3, CreateMasterEditionV3, create_metadata_accounts_v3};

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

pub fn derive_mint_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"mint_authority"], &crate::ID)
}

pub fn cpi_create_metadata_v3<'a>(
    payer: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    metadata: &AccountInfo<'a>,
    token_metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
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

    let cpi_accounts = anchor_spl::metadata::CreateMetadataAccountsV3 {
        metadata: metadata.to_account_info(),
        mint: mint.to_account_info(),
        mint_authority: mint_authority.to_account_info(),
        payer: payer.to_account_info(),
        update_authority: update_authority.to_account_info(),
        system_program: system_program.to_account_info(),
        rent: rent.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        token_metadata_program.to_account_info(),
        cpi_accounts,
    );

    create_metadata_accounts_v3(cpi_ctx, data, true, true, None)?;

    Ok(())
}

pub fn cpi_create_master_edition_v3<'a>(
    payer: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    metadata: &AccountInfo<'a>,
    master_edition: &AccountInfo<'a>,
    token_metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
) -> Result<()> {
    // Use Anchor SPL's create_master_edition_v3 instead of MPL's private version
    let cpi_accounts = CreateMasterEditionV3 {
        edition: master_edition.to_account_info(),
        mint: mint.to_account_info(),
        update_authority: update_authority.to_account_info(),
        mint_authority: mint_authority.to_account_info(),
        payer: payer.to_account_info(),
        metadata: metadata.to_account_info(),
        token_program: token_metadata_program.to_account_info(),
        system_program: system_program.to_account_info(),
        rent: rent.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        token_metadata_program.to_account_info(),
        cpi_accounts,
    );

    create_master_edition_v3(cpi_ctx, Some(0))?;

    Ok(())
}
