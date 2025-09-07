//! Metaplex Integration Module for Universal NFT Program
//! 
//! This module provides comprehensive CPI functions for creating and managing NFT metadata
//! using the official Metaplex Token Metadata program. It includes utilities for:
//! - Creating metadata accounts and master editions
//! - Collection verification and management
//! - PDA derivation and validation
//! - Error handling specific to Metaplex operations
//! - Converting between Universal NFT data and Metaplex formats

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed,
    instruction::{AccountMeta, Instruction},
};

// Import Metaplex Token Metadata types and constants
use mpl_token_metadata::{
    ID as TOKEN_METADATA_PROGRAM_ID,
    instructions::{
        CreateMetadataAccountV3,
        CreateMetadataAccountV3InstructionArgs,
        CreateMasterEditionV3,
        CreateMasterEditionV3InstructionArgs,
        UpdateMetadataAccountV2,
        UpdateMetadataAccountV2InstructionArgs,
        VerifyCollectionV1,
        VerifyCollectionV1InstructionArgs,
        UnverifyCollectionV1,
        UnverifyCollectionV1InstructionArgs,
        SetAndVerifyCollectionV1,
        SetAndVerifyCollectionV1InstructionArgs,
    },
    types::{
        DataV2, 
        Creator, 
        Collection as MetaplexCollection, 
        Uses, 
        UseMethod,
        CollectionDetails,
        Key,
        TokenStandard,
    },
    accounts::{
        Metadata,
        MasterEdition,
    },
};

use crate::state::{Collection, NftOrigin};
use crate::UniversalNftError;

/// Metaplex-specific error types for enhanced error handling
#[derive(Debug)]
pub enum MetaplexError {
    InvalidMetadataAccount,
    InvalidMasterEdition,
    CollectionVerificationFailed,
    MetadataCreationFailed,
    MasterEditionCreationFailed,
    InvalidCreatorData,
    InvalidCollectionData,
    PdaDerivationFailed,
    InsufficientFunds,
    InvalidUpdateAuthority,
    MetadataUpdateFailed,
}

impl From<MetaplexError> for anchor_lang::error::Error {
    fn from(error: MetaplexError) -> Self {
        msg!("Metaplex error: {:?}", error);
        match error {
            MetaplexError::InvalidMetadataAccount => crate::UniversalNftError::InvalidMetadata.into(),
            MetaplexError::InvalidMasterEdition => crate::UniversalNftError::InvalidMetadata.into(),
            MetaplexError::CollectionVerificationFailed => crate::UniversalNftError::InvalidCollection.into(),
            MetaplexError::MetadataCreationFailed => crate::UniversalNftError::InvalidMetadata.into(),
            MetaplexError::MasterEditionCreationFailed => crate::UniversalNftError::InvalidMetadata.into(),
            MetaplexError::InvalidCreatorData => crate::UniversalNftError::InvalidMetadata.into(),
            MetaplexError::InvalidCollectionData => crate::UniversalNftError::InvalidCollection.into(),
            MetaplexError::PdaDerivationFailed => crate::UniversalNftError::InvalidRecipient.into(),
            MetaplexError::InsufficientFunds => crate::UniversalNftError::InsufficientFunds.into(),
            MetaplexError::InvalidUpdateAuthority => crate::UniversalNftError::Unauthorized.into(),
            MetaplexError::MetadataUpdateFailed => crate::UniversalNftError::InvalidMetadata.into(),
        }
    }
}

/// Universal NFT metadata structure for conversion to Metaplex format
#[derive(Debug, Clone)]
pub struct UniversalNftMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<UniversalCreator>>,
    pub collection: Option<UniversalCollection>,
    pub uses: Option<UniversalUses>,
    pub is_mutable: bool,
    pub token_standard: Option<TokenStandard>,
}

#[derive(Debug, Clone)]
pub struct UniversalCreator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

#[derive(Debug, Clone)]
pub struct UniversalCollection {
    pub verified: bool,
    pub key: Pubkey,
}

#[derive(Debug, Clone)]
pub struct UniversalUses {
    pub use_method: UniversalUseMethod,
    pub remaining: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub enum UniversalUseMethod {
    Burn,
    Multiple,
    Single,
}

/// Create metadata account using proper Metaplex CPI
pub fn create_metadata_account_v3<'a>(
    metadata_account: &AccountInfo<'a>,
    mint_account: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    metadata: UniversalNftMetadata,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Validate input parameters
    validate_metadata_inputs(&metadata)?;
    
    // Convert Universal NFT metadata to Metaplex DataV2
    let data_v2 = convert_to_metaplex_data_v2(metadata.clone())?;
    
    // Create the instruction using official Metaplex instruction builder
    let create_metadata_ix = CreateMetadataAccountV3 {
        metadata: *metadata_account.key,
        mint: *mint_account.key,
        mint_authority: *mint_authority.key,
        payer: *payer.key,
        update_authority: *update_authority.key,
        system_program: *system_program.key,
        rent: Some(*rent.key),
    };

    let instruction_args = CreateMetadataAccountV3InstructionArgs {
        data: data_v2,
        is_mutable: metadata.is_mutable,
        collection_details: None, // Can be extended for collection-specific details
    };

    // Build the instruction
    let instruction = create_metadata_ix.instruction(instruction_args);

    // Execute CPI with comprehensive error handling
    invoke_signed(
        &instruction,
        &[
            metadata_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            update_authority.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to create metadata account: {:?}", e);
        MetaplexError::MetadataCreationFailed
    })?;

    msg!("Successfully created metadata account for mint: {}", mint_account.key);
    Ok(())
}

/// Create master edition using proper Metaplex CPI
pub fn create_master_edition_v3<'a>(
    master_edition_account: &AccountInfo<'a>,
    mint_account: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    metadata_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    max_supply: Option<u64>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Validate master edition parameters
    validate_master_edition_inputs(max_supply)?;
    
    // Create the instruction using official Metaplex instruction builder
    let create_master_edition_ix = CreateMasterEditionV3 {
        edition: *master_edition_account.key,
        mint: *mint_account.key,
        update_authority: *update_authority.key,
        mint_authority: *mint_authority.key,
        payer: *payer.key,
        metadata: *metadata_account.key,
        token_program: *token_program.key,
        system_program: *system_program.key,
        rent: Some(*rent.key),
    };

    let instruction_args = CreateMasterEditionV3InstructionArgs {
        max_supply,
    };

    // Build the instruction
    let instruction = create_master_edition_ix.instruction(instruction_args);

    // Execute CPI with comprehensive error handling
    invoke_signed(
        &instruction,
        &[
            master_edition_account.clone(),
            mint_account.clone(),
            update_authority.clone(),
            mint_authority.clone(),
            payer.clone(),
            metadata_account.clone(),
            token_program.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to create master edition: {:?}", e);
        MetaplexError::MasterEditionCreationFailed
    })?;

    msg!("Successfully created master edition for mint: {}", mint_account.key);
    Ok(())
}

/// Update metadata account using proper Metaplex CPI with field preservation
pub fn update_metadata_account_v2<'a>(
    metadata_account: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    new_metadata: Option<UniversalNftMetadata>,
    new_update_authority: Option<Pubkey>,
    primary_sale_happened: Option<bool>,
    is_mutable: Option<bool>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // If updating metadata, preserve existing fields and only update specified ones
    let new_data = if let Some(new_metadata) = new_metadata {
        // First, get existing metadata to preserve unspecified fields
        let existing_metadata = validate_metadata_account(metadata_account, &metadata_account.owner)?;
        
        // Create merged metadata preserving existing fields
        let merged_metadata = UniversalNftMetadata {
            name: if new_metadata.name.is_empty() { existing_metadata.name } else { new_metadata.name },
            symbol: if new_metadata.symbol.is_empty() { existing_metadata.symbol } else { new_metadata.symbol },
            uri: if new_metadata.uri.is_empty() { existing_metadata.uri } else { new_metadata.uri },
            seller_fee_basis_points: if new_metadata.seller_fee_basis_points == 0 { 
                existing_metadata.seller_fee_basis_points 
            } else { 
                new_metadata.seller_fee_basis_points 
            },
            creators: if new_metadata.creators.is_none() { 
                existing_metadata.creators.clone()
            } else { 
                new_metadata.creators 
            },
            collection: if new_metadata.collection.is_none() { 
                existing_metadata.collection.clone()
            } else { 
                new_metadata.collection 
            },
            uses: if new_metadata.uses.is_none() { 
                existing_metadata.uses.clone()
            } else { 
                new_metadata.uses 
            },
        };
        
        validate_metadata_inputs(&merged_metadata)?;
        Some(convert_to_metaplex_data_v2(merged_metadata)?)
    } else {
        None
    };
    
    // Create the instruction using official Metaplex instruction builder
    let update_metadata_ix = UpdateMetadataAccountV2 {
        metadata: *metadata_account.key,
        update_authority: *update_authority.key,
    };

    let instruction_args = UpdateMetadataAccountV2InstructionArgs {
        data: new_data,
        update_authority: new_update_authority,
        primary_sale_happened,
        is_mutable,
    };

    // Build the instruction
    let instruction = update_metadata_ix.instruction(instruction_args);

    // Execute CPI with error handling
    invoke_signed(
        &instruction,
        &[
            metadata_account.clone(),
            update_authority.clone(),
        ],
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to update metadata account: {:?}", e);
        MetaplexError::MetadataUpdateFailed
    })?;

    msg!("Successfully updated metadata account: {}", metadata_account.key);
    Ok(())
}

/// Verify collection for an NFT using proper Metaplex CPI
pub fn verify_collection<'a>(
    metadata_account: &AccountInfo<'a>,
    collection_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    collection_mint: &AccountInfo<'a>,
    collection_metadata: &AccountInfo<'a>,
    collection_master_edition: Option<&AccountInfo<'a>>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Create the instruction using official Metaplex instruction builder
    let verify_collection_ix = VerifyCollectionV1 {
        metadata: *metadata_account.key,
        collection_authority: *collection_authority.key,
        payer: *payer.key,
        collection_mint: *collection_mint.key,
        collection: *collection_metadata.key,
        collection_master_edition_account: collection_master_edition.map(|acc| acc.key()).copied(),
        collection_authority_record: None, // Can be extended for delegated authority
    };

    let instruction_args = VerifyCollectionV1InstructionArgs {};

    // Build the instruction
    let instruction = verify_collection_ix.instruction(instruction_args);

    // Prepare accounts for CPI
    let mut accounts = vec![
        metadata_account.clone(),
        collection_authority.clone(),
        payer.clone(),
        collection_mint.clone(),
        collection_metadata.clone(),
    ];
    
    if let Some(master_edition) = collection_master_edition {
        accounts.push(master_edition.clone());
    }

    // Execute CPI with error handling
    invoke_signed(
        &instruction,
        &accounts,
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to verify collection: {:?}", e);
        MetaplexError::CollectionVerificationFailed
    })?;

    msg!("Successfully verified collection for metadata: {}", metadata_account.key);
    Ok(())
}

/// Unverify collection for an NFT using proper Metaplex CPI
pub fn unverify_collection<'a>(
    metadata_account: &AccountInfo<'a>,
    collection_authority: &AccountInfo<'a>,
    collection_mint: &AccountInfo<'a>,
    collection_metadata: &AccountInfo<'a>,
    collection_master_edition: Option<&AccountInfo<'a>>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Create the instruction using official Metaplex instruction builder
    let unverify_collection_ix = UnverifyCollectionV1 {
        metadata: *metadata_account.key,
        collection_authority: *collection_authority.key,
        collection_mint: *collection_mint.key,
        collection: *collection_metadata.key,
        collection_master_edition_account: collection_master_edition.map(|acc| acc.key()).copied(),
        collection_authority_record: None,
    };

    let instruction_args = UnverifyCollectionV1InstructionArgs {};

    // Build the instruction
    let instruction = unverify_collection_ix.instruction(instruction_args);

    // Prepare accounts for CPI
    let mut accounts = vec![
        metadata_account.clone(),
        collection_authority.clone(),
        collection_mint.clone(),
        collection_metadata.clone(),
    ];
    
    if let Some(master_edition) = collection_master_edition {
        accounts.push(master_edition.clone());
    }

    // Execute CPI with error handling
    invoke_signed(
        &instruction,
        &accounts,
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to unverify collection: {:?}", e);
        MetaplexError::CollectionVerificationFailed
    })?;

    msg!("Successfully unverified collection for metadata: {}", metadata_account.key);
    Ok(())
}

/// Set and verify collection in one operation using proper Metaplex CPI
pub fn set_and_verify_collection<'a>(
    metadata_account: &AccountInfo<'a>,
    collection_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    collection_mint: &AccountInfo<'a>,
    collection_metadata: &AccountInfo<'a>,
    collection_master_edition: Option<&AccountInfo<'a>>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Create the instruction using official Metaplex instruction builder
    let set_and_verify_ix = SetAndVerifyCollectionV1 {
        metadata: *metadata_account.key,
        collection_authority: *collection_authority.key,
        payer: *payer.key,
        update_authority: *update_authority.key,
        collection_mint: *collection_mint.key,
        collection: *collection_metadata.key,
        collection_master_edition_account: collection_master_edition.map(|acc| acc.key()).copied(),
        collection_authority_record: None,
    };

    let instruction_args = SetAndVerifyCollectionV1InstructionArgs {};

    // Build the instruction
    let instruction = set_and_verify_ix.instruction(instruction_args);

    // Prepare accounts for CPI
    let mut accounts = vec![
        metadata_account.clone(),
        collection_authority.clone(),
        payer.clone(),
        update_authority.clone(),
        collection_mint.clone(),
        collection_metadata.clone(),
    ];
    
    if let Some(master_edition) = collection_master_edition {
        accounts.push(master_edition.clone());
    }

    // Execute CPI with error handling
    invoke_signed(
        &instruction,
        &accounts,
        signer_seeds,
    ).map_err(|e| {
        msg!("Failed to set and verify collection: {:?}", e);
        MetaplexError::CollectionVerificationFailed
    })?;

    msg!("Successfully set and verified collection for metadata: {}", metadata_account.key);
    Ok(())
}

/// Derive metadata PDA for a given mint
pub fn derive_metadata_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.as_ref(),
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    )
}

/// Derive master edition PDA for a given mint
pub fn derive_master_edition_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.as_ref(),
            b"edition",
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    )
}

/// Derive collection authority record PDA
pub fn derive_collection_authority_record_pda(
    collection_mint: &Pubkey,
    authority: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            collection_mint.as_ref(),
            b"collection_authority",
            authority.as_ref(),
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    )
}

/// Validate metadata account structure
pub fn validate_metadata_account(
    metadata_account: &AccountInfo,
    expected_mint: &Pubkey,
) -> Result<Metadata> {
    // Check if account is owned by the metadata program
    require!(
        metadata_account.owner == &TOKEN_METADATA_PROGRAM_ID,
        MetaplexError::InvalidMetadataAccount
    );

    // Verify the PDA derivation
    let (expected_metadata_pda, _) = derive_metadata_pda(expected_mint);
    require!(
        metadata_account.key == &expected_metadata_pda,
        MetaplexError::PdaDerivationFailed
    );

    // Deserialize and validate metadata
    let metadata = Metadata::from_account_info(metadata_account)
        .map_err(|_| MetaplexError::InvalidMetadataAccount)?;

    // Verify mint matches
    require!(
        metadata.mint == *expected_mint,
        MetaplexError::InvalidMetadataAccount
    );

    Ok(metadata)
}

/// Validate master edition account structure
pub fn validate_master_edition_account(
    master_edition_account: &AccountInfo,
    expected_mint: &Pubkey,
) -> Result<MasterEdition> {
    // Check if account is owned by the metadata program
    require!(
        master_edition_account.owner == &TOKEN_METADATA_PROGRAM_ID,
        MetaplexError::InvalidMasterEdition
    );

    // Verify the PDA derivation
    let (expected_edition_pda, _) = derive_master_edition_pda(expected_mint);
    require!(
        master_edition_account.key == &expected_edition_pda,
        MetaplexError::PdaDerivationFailed
    );

    // Deserialize and validate master edition
    let master_edition = MasterEdition::from_account_info(master_edition_account)
        .map_err(|_| MetaplexError::InvalidMasterEdition)?;

    Ok(master_edition)
}

/// Convert Universal NFT Collection to Metaplex metadata format
pub fn collection_to_metaplex_metadata(
    collection: &Collection,
    mint: &Pubkey,
    authority: &Pubkey,
) -> UniversalNftMetadata {
    UniversalNftMetadata {
        name: collection.name.clone(),
        symbol: collection.symbol.clone(),
        uri: collection.uri.clone(),
        seller_fee_basis_points: 0, // No royalties for Universal NFT collections
        creators: Some(vec![UniversalCreator {
            address: *authority,
            verified: true,
            share: 100,
        }]),
        collection: None, // Collections don't have parent collections
        uses: None,
        is_mutable: true, // Allow updates for cross-chain scenarios
        token_standard: Some(TokenStandard::NonFungible),
    }
}

/// Convert NFT Origin to Metaplex metadata format
pub fn nft_origin_to_metaplex_metadata(
    nft_origin: &NftOrigin,
    collection: &Collection,
    token_id: u64,
    authority: &Pubkey,
) -> UniversalNftMetadata {
    let name = if !nft_origin.metadata_uri.is_empty() {
        format!("Universal NFT #{}", token_id)
    } else {
        format!("{} #{}", collection.name, token_id)
    };

    UniversalNftMetadata {
        name,
        symbol: collection.symbol.clone(),
        uri: if !nft_origin.metadata_uri.is_empty() {
            nft_origin.metadata_uri.clone()
        } else {
            collection.uri.clone()
        },
        seller_fee_basis_points: 0,
        creators: Some(vec![UniversalCreator {
            address: *authority,
            verified: true,
            share: 100,
        }]),
        collection: Some(UniversalCollection {
            verified: false, // Will be verified separately
            key: nft_origin.collection,
        }),
        uses: None,
        is_mutable: true, // Allow updates for cross-chain metadata changes
        token_standard: Some(TokenStandard::NonFungible),
    }
}

/// Create complete NFT with metadata and master edition
pub fn create_complete_nft<'a>(
    mint_account: &AccountInfo<'a>,
    metadata_account: &AccountInfo<'a>,
    master_edition_account: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    metadata: UniversalNftMetadata,
    max_supply: Option<u64>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Create metadata account
    create_metadata_account_v3(
        metadata_account,
        mint_account,
        mint_authority,
        payer,
        update_authority,
        system_program,
        rent,
        metadata,
        signer_seeds,
    )?;

    // Create master edition
    create_master_edition_v3(
        master_edition_account,
        mint_account,
        update_authority,
        mint_authority,
        payer,
        metadata_account,
        token_program,
        system_program,
        rent,
        max_supply,
        signer_seeds,
    )?;

    msg!("Successfully created complete NFT with metadata and master edition");
    Ok(())
}

/// Helper function to calculate required lamports for metadata account
pub fn calculate_metadata_rent() -> Result<u64> {
    let rent = Rent::get()?;
    // Metadata account size is approximately 679 bytes
    Ok(rent.minimum_balance(679))
}

/// Helper function to calculate required lamports for master edition account
pub fn calculate_master_edition_rent() -> Result<u64> {
    let rent = Rent::get()?;
    // Master edition account size is approximately 282 bytes
    Ok(rent.minimum_balance(282))
}

/// Validate metadata inputs before creating Metaplex accounts
fn validate_metadata_inputs(metadata: &UniversalNftMetadata) -> Result<()> {
    // Validate name
    require!(
        !metadata.name.is_empty() && metadata.name.len() <= 32,
        MetaplexError::InvalidMetadataAccount
    );

    // Validate symbol
    require!(
        !metadata.symbol.is_empty() && metadata.symbol.len() <= 10,
        MetaplexError::InvalidMetadataAccount
    );

    // Validate URI
    require!(
        !metadata.uri.is_empty() && metadata.uri.len() <= 200,
        MetaplexError::InvalidMetadataAccount
    );

    // Validate seller fee basis points (max 10000 = 100%)
    require!(
        metadata.seller_fee_basis_points <= 10000,
        MetaplexError::InvalidMetadataAccount
    );

    // Validate creators if present
    if let Some(creators) = &metadata.creators {
        require!(
            !creators.is_empty() && creators.len() <= 5,
            MetaplexError::InvalidCreatorData
        );

        let total_share: u16 = creators.iter().map(|c| c.share as u16).sum();
        require!(
            total_share == 100,
            MetaplexError::InvalidCreatorData
        );
    }

    Ok(())
}

/// Validate master edition inputs
fn validate_master_edition_inputs(max_supply: Option<u64>) -> Result<()> {
    // Validate max supply if provided
    if let Some(supply) = max_supply {
        require!(
            supply <= u64::MAX / 2,
            MetaplexError::InvalidMasterEdition
        );
    }

    Ok(())
}

/// Convert Universal NFT metadata to Metaplex DataV2
fn convert_to_metaplex_data_v2(metadata: UniversalNftMetadata) -> Result<DataV2> {
    let creators = metadata.creators.map(|creators| {
        creators.into_iter().map(|creator| Creator {
            address: creator.address,
            verified: creator.verified,
            share: creator.share,
        }).collect()
    });

    let collection = metadata.collection.map(|collection| MetaplexCollection {
        verified: collection.verified,
        key: collection.key,
    });

    let uses = metadata.uses.map(|uses| Uses {
        use_method: match uses.use_method {
            UniversalUseMethod::Burn => UseMethod::Burn,
            UniversalUseMethod::Multiple => UseMethod::Multiple,
            UniversalUseMethod::Single => UseMethod::Single,
        },
        remaining: uses.remaining,
        total: uses.total,
    });

    Ok(DataV2 {
        name: metadata.name,
        symbol: metadata.symbol,
        uri: metadata.uri,
        seller_fee_basis_points: metadata.seller_fee_basis_points,
        creators,
        collection,
        uses,
    })
}

/// Check if an account is a valid Metaplex metadata account
pub fn is_metadata_account(account: &AccountInfo) -> bool {
    account.owner == &TOKEN_METADATA_PROGRAM_ID && 
    account.data_len() >= 1 && 
    account.try_borrow_data().map_or(false, |data| {
        data.len() > 0 && data[0] == Key::MetadataV1 as u8
    })
}

/// Check if an account is a valid Metaplex master edition account
pub fn is_master_edition_account(account: &AccountInfo) -> bool {
    account.owner == &TOKEN_METADATA_PROGRAM_ID && 
    account.data_len() >= 1 && 
    account.try_borrow_data().map_or(false, |data| {
        data.len() > 0 && (data[0] == Key::MasterEditionV1 as u8 || data[0] == Key::MasterEditionV2 as u8)
    })
}

/// Get metadata account info from mint
pub fn get_metadata_info_from_mint(mint: &Pubkey) -> (Pubkey, u8) {
    derive_metadata_pda(mint)
}

/// Get master edition account info from mint
pub fn get_master_edition_info_from_mint(mint: &Pubkey) -> (Pubkey, u8) {
    derive_master_edition_pda(mint)
}

/// Enhanced error handling for Metaplex operations
pub fn handle_metaplex_error(error: anchor_lang::error::Error) -> MetaplexError {
    msg!("Metaplex operation failed: {:?}", error);
    
    // Map specific Anchor/Solana errors to Metaplex errors
    match error {
        anchor_lang::error::Error::AccountNotEnoughKeys => MetaplexError::InvalidMetadataAccount,
        anchor_lang::error::Error::AccountNotMutable => MetaplexError::InvalidUpdateAuthority,
        anchor_lang::error::Error::InstructionMissing => MetaplexError::MetadataCreationFailed,
        _ => MetaplexError::MetadataCreationFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_validation() {
        let metadata = UniversalNftMetadata {
            name: "Test NFT".to_string(),
            symbol: "TEST".to_string(),
            uri: "https://example.com/metadata.json".to_string(),
            seller_fee_basis_points: 500,
            creators: Some(vec![UniversalCreator {
                address: Pubkey::default(),
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
            is_mutable: true,
            token_standard: Some(TokenStandard::NonFungible),
        };

        assert!(validate_metadata_inputs(&metadata).is_ok());
    }

    #[test]
    fn test_pda_derivation() {
        let mint = Pubkey::new_unique();
        let (metadata_pda, _) = derive_metadata_pda(&mint);
        let (edition_pda, _) = derive_master_edition_pda(&mint);
        
        assert_ne!(metadata_pda, edition_pda);
        assert_ne!(metadata_pda, Pubkey::default());
        assert_ne!(edition_pda, Pubkey::default());
    }
}