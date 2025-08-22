use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    account_info::AccountInfo,
};
use crate::error::UniversalNftError;
use crate::util::constants::DEFAULT_SELLER_FEE_BASIS_POINTS;
use mpl_token_metadata::instructions::{
    CreateMetadataAccountV3Builder,
    CreateMasterEditionV3Builder,
};
use mpl_token_metadata::types::DataV2;

/// Create metadata account using CPI to Metaplex Token Metadata program
pub fn create_metadata_account<'a>(
    metadata: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    name: &str,
    symbol: &str,
    uri: &str,
    authority_signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    // Build CreateMetadataAccountV3 using builder API
    let data = DataV2 {
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        seller_fee_basis_points: DEFAULT_SELLER_FEE_BASIS_POINTS,
        creators: None,
        collection: None,
        uses: None,
    };

    let ix = CreateMetadataAccountV3Builder::new()
        .metadata(*metadata.key)
        .mint(*mint.key)
        .mint_authority(*authority.key)
        .payer(*payer.key)
        .update_authority(*authority.key, true)
        .system_program(*system_program.key)
        .rent(Some(*rent.key))
        .data(data)
        .is_mutable(true)
        .instruction();

    match authority_signer_seeds {
        Some(seeds) => anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                metadata.clone(),
                mint.clone(),
                authority.clone(),
                payer.clone(),
                system_program.clone(),
                rent.clone(),
            ],
            seeds,
        ).map_err(|_| UniversalNftError::MetadataCreationFailed)?,
        None => invoke(
            &ix,
            &[
                metadata.clone(),
                mint.clone(),
                authority.clone(),
                payer.clone(),
                system_program.clone(),
                rent.clone(),
            ],
        ).map_err(|_| UniversalNftError::MetadataCreationFailed)?,
    }

    Ok(())
}

/// Create master edition account using CPI to Metaplex Token Metadata program
pub fn create_master_edition_account<'a>(
    master_edition: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    metadata: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    authority_signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    // Build CreateMasterEditionV3 using builder API
    let ix = CreateMasterEditionV3Builder::new()
        .edition(*master_edition.key)
        .mint(*mint.key)
        .update_authority(*authority.key)
        .mint_authority(*authority.key)
        .payer(*payer.key)
        .metadata(*metadata.key)
        .token_program(*token_program.key)
        .system_program(*system_program.key)
        .rent(Some(*rent.key))
        .max_supply(0u64)
        .instruction();

    match authority_signer_seeds {
        Some(seeds) => anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                master_edition.clone(),
                mint.clone(),
                authority.clone(),
                payer.clone(),
                metadata.clone(),
                token_program.clone(),
                system_program.clone(),
                rent.clone(),
            ],
            seeds,
        ).map_err(|_| UniversalNftError::MasterEditionCreationFailed)?,
        None => invoke(
            &ix,
            &[
                master_edition.clone(),
                mint.clone(),
                authority.clone(),
                payer.clone(),
                metadata.clone(),
                token_program.clone(),
                system_program.clone(),
                rent.clone(),
            ],
        ).map_err(|_| UniversalNftError::MasterEditionCreationFailed)?,
    }

    Ok(())
}
