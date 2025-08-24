use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    account_info::AccountInfo,
};
use crate::errors::Errors;
use mpl_token_metadata::instructions::{
    CreateMetadataAccountV3Builder,
    CreateMasterEditionV3Builder,
};
use mpl_token_metadata::types::DataV2;

/// Initialize metadata account using Metaplex Token Metadata program
pub fn initialize_metadata_account<'a>(
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
        seller_fee_basis_points: 0,
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

    // Account order must match the builder's metas exactly
    // [metadata, mint, mint_authority, payer, update_authority, system_program, rent]
    match authority_signer_seeds {
        Some(seeds) => anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                metadata.clone(),        // metadata
                mint.clone(),            // mint
                authority.clone(),       // mint_authority
                payer.clone(),           // payer
                authority.clone(),       // update_authority (same pubkey, duplicated)
                system_program.clone(),  // system_program
                rent.clone(),            // rent
            ],
            seeds,
        ).map_err(|_| Errors::InvalidDataFormat)?,
        None => invoke(
            &ix,
            &[
                metadata.clone(),        // metadata
                mint.clone(),            // mint
                authority.clone(),       // mint_authority
                payer.clone(),           // payer
                authority.clone(),       // update_authority (same pubkey, duplicated)
                system_program.clone(),  // system_program
                rent.clone(),            // rent
            ],
        ).map_err(|_| Errors::InvalidDataFormat)?,
    }

    Ok(())
}

/// Initialize master edition account using Metaplex Token Metadata program
pub fn initialize_master_edition_account<'a>(
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

    // Account order must match the builder's metas exactly
    // [edition, mint, update_authority, mint_authority, payer, metadata, token_program, system_program, rent]
    match authority_signer_seeds {
        Some(seeds) => anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                master_edition.clone(),   // edition
                mint.clone(),             // mint
                authority.clone(),        // update_authority
                authority.clone(),        // mint_authority (same pubkey, duplicated)
                payer.clone(),            // payer
                metadata.clone(),         // metadata
                token_program.clone(),    // token_program
                system_program.clone(),   // system_program
                rent.clone(),             // rent
            ],
            seeds,
        ).map_err(|_| Errors::InvalidDataFormat)?,
        None => invoke(
            &ix,
            &[
                master_edition.clone(),   // edition
                mint.clone(),             // mint
                authority.clone(),        // update_authority
                authority.clone(),        // mint_authority (same pubkey, duplicated)
                payer.clone(),            // payer
                metadata.clone(),         // metadata
                token_program.clone(),    // token_program
                system_program.clone(),   // system_program
                rent.clone(),             // rent
            ],
        ).map_err(|_| Errors::InvalidDataFormat)?,
    }

    Ok(())
}
