use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    keccak,
};

/// Utility functions for handling Solana-specific challenges

/// Request additional compute units to handle complex operations
/// NOTE: Compute budget must be set client-side at transaction creation.
/// Clients should include ComputeBudgetInstruction::set_compute_unit_limit(units)
/// as the first instruction in their transaction.
pub fn request_compute_units(_units: u32) -> Result<()> {
    // No-op: compute budget cannot be modified via CPI
    // This must be done by the client when constructing the transaction
    Ok(())
}

/// Calculate rent exemption for account creation
pub fn calculate_rent_exemption(account_size: usize, rent: &Rent) -> u64 {
    rent.minimum_balance(account_size)
}

/// Validate Solana address format
pub fn validate_solana_address(address_bytes: &[u8]) -> Result<Pubkey> {
    if address_bytes.len() != 32 {
        return Err(error!(crate::errors::UniversalNftError::InvalidRecipient));
    }
    
    Pubkey::try_from(address_bytes)
        .map_err(|_| error!(crate::errors::UniversalNftError::InvalidRecipient))
}

/// Generate deterministic token ID based on collection and metadata
/// Uses keccak256 for cross-platform deterministic hashing
pub fn generate_token_id(collection: &Pubkey, next_id: u64, uri: &str) -> u64 {
    // Create a canonical byte sequence
    let mut data = Vec::new();
    data.extend_from_slice(&collection.to_bytes());
    data.extend_from_slice(&next_id.to_be_bytes()); // Big-endian for consistency
    data.extend_from_slice(uri.as_bytes());
    
    // Compute keccak256 hash
    let hash = keccak::hash(&data);
    
    // Take first 8 bytes as u64 (big-endian)
    u64::from_be_bytes([
        hash.0[0], hash.0[1], hash.0[2], hash.0[3],
        hash.0[4], hash.0[5], hash.0[6], hash.0[7]
    ])
}

/// Validate URI format and length
pub fn validate_uri(uri: &str) -> Result<()> {
    if uri.is_empty() || uri.len() > 200 {
        return Err(error!(crate::errors::UniversalNftError::InvalidMessage));
    }
    
    // Basic URI validation - should start with http/https or be IPFS
    if !uri.starts_with("http://") && 
       !uri.starts_with("https://") && 
       !uri.starts_with("ipfs://") &&
       !uri.starts_with("ar://") {
        return Err(error!(crate::errors::UniversalNftError::InvalidMessage));
    }
    
    Ok(())
}

/// Helper to create associated token account instruction
pub fn create_associated_token_account_ix(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    spl_associated_token_account::instruction::create_associated_token_account(
        payer,
        owner,
        mint,
        &spl_token::id(),
    )
}

/// Check if account has sufficient rent exemption
pub fn check_rent_exemption(account_info: &AccountInfo, required_size: usize, rent: &Rent) -> Result<()> {
    let required_lamports = rent.minimum_balance(required_size);
    
    if account_info.lamports() < required_lamports {
        return Err(error!(crate::errors::UniversalNftError::RentExemptionFailed));
    }
    
    Ok(())
}
