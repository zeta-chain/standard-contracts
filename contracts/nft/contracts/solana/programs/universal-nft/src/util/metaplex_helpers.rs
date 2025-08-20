use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    account_info::AccountInfo,
};
use crate::error::UniversalNftError;
use crate::util::constants::{TOKEN_METADATA_PROGRAM_ID, DEFAULT_SELLER_FEE_BASIS_POINTS};
use mpl_token_metadata::instruction::{create_metadata_accounts_v3, create_master_edition_v3};

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
    // Build CreateMetadataAccountsV3 using mpl-token-metadata
    let create_metadata_account_v3_ix = create_metadata_accounts_v3(
        TOKEN_METADATA_PROGRAM_ID,
        *metadata.key,
        *mint.key,
        *authority.key,           // mint_authority
        *payer.key,               // payer
        *authority.key,           // update_authority
        name.to_string(),
        symbol.to_string(),
        uri.to_string(),
        None,                     // creators
        DEFAULT_SELLER_FEE_BASIS_POINTS,
        true,                     // is_mutable
        None,                     // collection details
        None,                     // uses
        None,                     // collection
    );

    match authority_signer_seeds {
        Some(seeds) => anchor_lang::solana_program::program::invoke_signed(
            &create_metadata_account_v3_ix,
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
            &create_metadata_account_v3_ix,
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
    // Build CreateMasterEditionV3 using mpl-token-metadata
    let create_master_edition_v3_ix = create_master_edition_v3(
        TOKEN_METADATA_PROGRAM_ID,
        *master_edition.key,
        *mint.key,
        *authority.key, // update_authority
        *authority.key, // mint_authority (same authority here)
        *payer.key,
        *metadata.key,
        *token_program.key,
        *system_program.key,
        *rent.key,
        None, // max_supply
    );

    match authority_signer_seeds {
        Some(seeds) => anchor_lang::solana_program::program::invoke_signed(
            &create_master_edition_v3_ix,
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
            &create_master_edition_v3_ix,
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
