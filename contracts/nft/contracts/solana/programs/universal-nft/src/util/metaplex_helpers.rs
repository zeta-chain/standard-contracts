use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
};
use crate::error::UniversalNftError;
use crate::util::constants::{TOKEN_METADATA_PROGRAM_ID, DEFAULT_SELLER_FEE_BASIS_POINTS};

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
) -> Result<()> {
    // Create metadata account instruction data
    let create_metadata_account_v3_ix = Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(metadata.key(), false),
            AccountMeta::new_readonly(mint.key(), false),
            AccountMeta::new_readonly(authority.key(), true),
            AccountMeta::new(payer.key(), true),
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(system_program.key(), false),
            AccountMeta::new_readonly(rent.key(), false),
        ],
        data: {
            let mut data = vec![251, 52, 168, 105, 238, 247, 42, 129]; // CreateMetadataAccountV3 discriminator
            
            // Data account struct
            data.extend_from_slice(&(name.len() as u32).to_le_bytes());
            data.extend_from_slice(name.as_bytes());
            data.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
            data.extend_from_slice(symbol.as_bytes());
            data.extend_from_slice(&(uri.len() as u32).to_le_bytes());
            data.extend_from_slice(uri.as_bytes());
            
            // Creators (empty)
            data.extend_from_slice(&[0]); // Option::None for creators
            
            // Seller fee basis points
            data.extend_from_slice(&DEFAULT_SELLER_FEE_BASIS_POINTS.to_le_bytes());
            
            // Is mutable
            data.extend_from_slice(&[1]); // true
            
            // Collection (empty)
            data.extend_from_slice(&[0]); // Option::None for collection
            
            // Uses (empty)
            data.extend_from_slice(&[0]); // Option::None for uses
            
            data
        },
    };

    invoke(
        &create_metadata_account_v3_ix,
        &[
            metadata.clone(),
            mint.clone(),
            authority.clone(),
            payer.clone(),
            system_program.clone(),
            rent.clone(),
        ],
    ).map_err(|_| UniversalNftError::MetadataCreationFailed)?;

    Ok(())
}

/// Create master edition account using CPI to Metaplex Token Metadata program
pub fn create_master_edition_account<'a>(
    master_edition: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    metadata: &AccountInfo<'a>,
    metadata_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
) -> Result<()> {
    let create_master_edition_v3_ix = Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(master_edition.key(), false),
            AccountMeta::new_readonly(mint.key(), false),
            AccountMeta::new_readonly(authority.key(), true),
            AccountMeta::new(payer.key(), true),
            AccountMeta::new_readonly(metadata.key(), false),
            AccountMeta::new_readonly(TOKEN_METADATA_PROGRAM_ID, false),
            AccountMeta::new_readonly(system_program.key(), false),
            AccountMeta::new_readonly(rent.key(), false),
        ],
        data: {
            let mut data = vec![166, 246, 188, 207, 127, 197, 12, 70]; // CreateMasterEditionV3 discriminator
            
            // Max supply (None for unlimited)
            data.extend_from_slice(&[0]); // Option::None
            
            data
        },
    };

    invoke(
        &create_master_edition_v3_ix,
        &[
            master_edition.clone(),
            mint.clone(),
            authority.clone(),
            payer.clone(),
            metadata.clone(),
            metadata_program.clone(),
            system_program.clone(),
            rent.clone(),
        ],
    ).map_err(|_| UniversalNftError::MasterEditionCreationFailed)?;

    Ok(())
}
