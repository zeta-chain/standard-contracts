use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    program::invoke,
};

/// Utility functions for handling Solana-specific challenges

/// Request additional compute units to handle complex operations
pub fn request_compute_units(units: u32) -> Result<()> {
    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(units);
    
    invoke(
        &compute_budget_ix,
        &[],
    ).map_err(|_| error!(crate::errors::UniversalNftError::InsufficientComputeBudget))?;
    
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
pub fn generate_token_id(collection: &Pubkey, next_id: u64, uri: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    collection.hash(&mut hasher);
    next_id.hash(&mut hasher);
    uri.hash(&mut hasher);
    
    hasher.finish()
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
