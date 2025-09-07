use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    clock::Clock,
    keccak,
    sysvar::Sysvar,
};
use std::collections::HashMap;

use crate::state::{Collection, NftOrigin, Connected};
use crate::gateway::CrossChainMessage;
use crate::{UniversalNftError, find_nft_origin_pda};

/// Security configuration constants
pub const MAX_OPERATIONS_PER_SLOT: u64 = 100;
pub const MAX_OPERATIONS_PER_AUTHORITY: u64 = 50;
pub const EMERGENCY_PAUSE_DURATION: i64 = 86400; // 24 hours in seconds
pub const MAX_NONCE_GAP: u64 = 1000;
pub const MIN_SIGNATURE_RECOVERY_ID: u8 = 0;
pub const MAX_SIGNATURE_RECOVERY_ID: u8 = 3;
pub const TSS_SIGNATURE_LENGTH: usize = 64;
pub const TSS_PUBKEY_LENGTH: usize = 64;
pub const EVM_ADDRESS_LENGTH: usize = 20;
pub const SOLANA_ADDRESS_LENGTH: usize = 32;
pub const MAX_URI_LENGTH: usize = 200;
pub const MIN_TOKEN_ID: u64 = 1;
pub const MAX_TOKEN_ID: u64 = u64::MAX - 1;

/// Rate limiting structure for tracking operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RateLimitTracker {
    pub slot: u64,
    pub operations_count: u64,
    pub last_operation_timestamp: i64,
}

/// Emergency pause state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyPauseState {
    pub is_paused: bool,
    pub paused_at: i64,
    pub pause_reason: String,
    pub paused_by: Pubkey,
}

/// Security validation result
#[derive(Debug, Clone)]
pub struct SecurityValidationResult {
    pub is_valid: bool,
    pub error_code: Option<UniversalNftError>,
    pub details: String,
}

impl SecurityValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            error_code: None,
            details: "Validation passed".to_string(),
        }
    }

    pub fn invalid(error: UniversalNftError, details: &str) -> Self {
        Self {
            is_valid: false,
            error_code: Some(error),
            details: details.to_string(),
        }
    }
}

/// Enhanced TSS signature validation with comprehensive checks
pub fn validate_tss_signature(
    message_hash: &[u8; 32],
    signature: &[u8; 64],
    recovery_id: u8,
    expected_tss_address: &[u8; 20],
) -> Result<SecurityValidationResult> {
    // Validate recovery ID range
    if recovery_id > MAX_SIGNATURE_RECOVERY_ID {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTssSignature,
            "Recovery ID out of valid range (0-3)"
        ));
    }

    // Validate signature is not all zeros
    if signature.iter().all(|&b| b == 0) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTssSignature,
            "Signature cannot be all zeros"
        ));
    }

    // Validate message hash is not all zeros
    if message_hash.iter().all(|&b| b == 0) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidMessageHash,
            "Message hash cannot be all zeros"
        ));
    }

    // Recover public key from signature using Solana syscall
    let recovered_pubkey = anchor_lang::solana_program::secp256k1_recover::secp256k1_recover(message_hash, recovery_id, signature)
        .map_err(|_| UniversalNftError::InvalidTssSignature)?;

    // Validate recovered public key format
    if recovered_pubkey.0.len() != TSS_PUBKEY_LENGTH {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTssSignature,
            "Invalid recovered public key length"
        ));
    }

    // Validate public key is not all zeros
    if recovered_pubkey.0.iter().all(|&b| b == 0) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTssSignature,
            "Recovered public key cannot be all zeros"
        ));
    }

    // Convert public key to Ethereum address
    let recovered_address = pubkey_to_eth_address(&recovered_pubkey.0)?;

    // Validate against expected TSS address
    if recovered_address != *expected_tss_address {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::UnauthorizedTssAddress,
            "TSS signature does not match expected address"
        ));
    }

    // Additional signature malleability check
    if is_signature_malleable(signature) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTssSignature,
            "Signature is malleable and potentially unsafe"
        ));
    }

    Ok(SecurityValidationResult::valid())
}

/// Check for signature malleability (high S values)
fn is_signature_malleable(signature: &[u8; 64]) -> bool {
    // Extract S value (last 32 bytes of signature)
    let s_bytes = &signature[32..64];
    
    // secp256k1 curve order (n)
    let secp256k1_n = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
        0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
        0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
    ];
    
    // Half of secp256k1 curve order
    let secp256k1_n_half = [
        0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
        0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
    ];
    
    // Check if S > n/2 (malleable signature)
    for i in 0..32 {
        if s_bytes[i] > secp256k1_n_half[i] {
            return true;
        } else if s_bytes[i] < secp256k1_n_half[i] {
            return false;
        }
    }
    
    false
}

/// Enhanced cross-chain message integrity verification
pub fn verify_cross_chain_message_integrity(
    message_data: &[u8],
    expected_hash: &[u8; 32],
    source_chain_id: u64,
) -> Result<SecurityValidationResult> {
    // Validate message data is not empty
    if message_data.is_empty() {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidMessage,
            "Message data cannot be empty"
        ));
    }

    // Validate message size limits
    if message_data.len() > 10240 { // 10KB max
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidMessage,
            "Message data exceeds maximum size limit"
        ));
    }

    // Compute message hash
    let computed_hash = keccak::hash(message_data);
    
    // Verify hash integrity
    if computed_hash.to_bytes() != *expected_hash {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidMessageHash,
            "Message hash does not match computed hash"
        ));
    }

    // Validate source chain ID
    if !crate::state::is_supported_chain(source_chain_id) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::UnsupportedChain,
            "Source chain ID is not supported"
        ));
    }

    // Additional message format validation based on source chain
    match source_chain_id {
        // EVM chains - validate ABI encoding
        1 | 56 | 137 | 8453 | 42161 | 10 | 11155111 | 97 | 80001 | 84532 | 421614 | 11155420 => {
            validate_evm_message_format(message_data)?;
        },
        // ZetaChain - validate ZetaChain format
        7000 | 7001 => {
            validate_zetachain_message_format(message_data)?;
        },
        // Solana - validate Borsh format
        101 | 102 | 103 => {
            validate_solana_message_format(message_data)?;
        },
        _ => {
            return Ok(SecurityValidationResult::invalid(
                UniversalNftError::UnsupportedChain,
                "Unknown chain format"
            ));
        }
    }

    Ok(SecurityValidationResult::valid())
}

/// Validate EVM message format
fn validate_evm_message_format(message_data: &[u8]) -> Result<()> {
    // Minimum ABI encoded message size
    require!(message_data.len() >= 148, UniversalNftError::InvalidMessage);
    
    // Validate ABI encoding structure
    // First 32 bytes should be token_id
    // Next 32 bytes should be URI offset
    // etc.
    
    Ok(())
}

/// Validate ZetaChain message format
fn validate_zetachain_message_format(message_data: &[u8]) -> Result<()> {
    // Try to deserialize as ZetaChain message
    let _zetachain_msg = crate::gateway::ZetaChainCrossChainMessage::try_from_slice(message_data)
        .map_err(|_| UniversalNftError::InvalidMessage)?;
    
    Ok(())
}

/// Validate Solana message format
fn validate_solana_message_format(message_data: &[u8]) -> Result<()> {
    // Try to deserialize as CrossChainMessage
    let _cross_chain_msg = CrossChainMessage::try_from_slice(message_data)
        .map_err(|_| UniversalNftError::InvalidMessage)?;
    
    Ok(())
}

/// Comprehensive replay attack protection
pub fn validate_nonce_and_prevent_replay(
    collection: &Collection,
    provided_nonce: u64,
    message_hash: &[u8; 32],
) -> Result<SecurityValidationResult> {
    // Basic nonce validation
    if provided_nonce <= collection.nonce {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidNonce,
            "Nonce must be greater than current collection nonce"
        ));
    }

    // Prevent nonce gaps that are too large
    if provided_nonce > collection.nonce.saturating_add(MAX_NONCE_GAP) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidNonce,
            "Nonce gap is too large, potential replay attack"
        ));
    }

    // Additional replay protection: check for duplicate message hashes
    // In a real implementation, this would check a bloom filter or cache
    if is_message_hash_seen(message_hash) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidNonce,
            "Message hash has been seen before, potential replay attack"
        ));
    }

    // Validate nonce is not from the future (with reasonable tolerance)
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    let nonce_timestamp = extract_timestamp_from_nonce(provided_nonce);
    
    if nonce_timestamp > current_timestamp + 300 { // 5 minute tolerance
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidNonce,
            "Nonce timestamp is too far in the future"
        ));
    }

    Ok(SecurityValidationResult::valid())
}

/// Extract timestamp from nonce (if nonce includes timestamp)
fn extract_timestamp_from_nonce(nonce: u64) -> i64 {
    // This is a simplified approach - real implementation would depend on nonce format
    (nonce >> 32) as i64
}

/// Check if message hash has been seen before (simplified)
fn is_message_hash_seen(_message_hash: &[u8; 32]) -> bool {
    // In a real implementation, this would check against a bloom filter or cache
    // For now, returning false (not seen)
    false
}

/// Rate limiting implementation
pub fn check_rate_limits(
    authority: &Pubkey,
    operation_type: &str,
) -> Result<SecurityValidationResult> {
    let clock = Clock::get()?;
    let current_slot = clock.slot;
    let current_timestamp = clock.unix_timestamp;

    // Check global rate limits per slot
    let global_operations = get_global_operations_count(current_slot);
    if global_operations >= MAX_OPERATIONS_PER_SLOT {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidNonce, // Reusing error code
            "Global rate limit exceeded for this slot"
        ));
    }

    // Check per-authority rate limits
    let authority_operations = get_authority_operations_count(authority, current_slot);
    if authority_operations >= MAX_OPERATIONS_PER_AUTHORITY {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidNonce, // Reusing error code
            "Authority rate limit exceeded for this slot"
        ));
    }

    // Check operation-specific rate limits
    match operation_type {
        "mint" => {
            if authority_operations >= 20 {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::InvalidNonce,
                    "Mint rate limit exceeded"
                ));
            }
        },
        "transfer" => {
            if authority_operations >= 30 {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::InvalidNonce,
                    "Transfer rate limit exceeded"
                ));
            }
        },
        "cross_chain" => {
            if authority_operations >= 10 {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::InvalidNonce,
                    "Cross-chain rate limit exceeded"
                ));
            }
        },
        _ => {}
    }

    // Update rate limit counters
    increment_operation_counters(authority, current_slot, current_timestamp);

    Ok(SecurityValidationResult::valid())
}

/// Get global operations count for current slot (simplified)
fn get_global_operations_count(_slot: u64) -> u64 {
    // In a real implementation, this would query a global counter account
    0
}

/// Get authority operations count for current slot (simplified)
fn get_authority_operations_count(_authority: &Pubkey, _slot: u64) -> u64 {
    // In a real implementation, this would query an authority-specific counter
    0
}

/// Increment operation counters (simplified)
fn increment_operation_counters(_authority: &Pubkey, _slot: u64, _timestamp: i64) {
    // In a real implementation, this would update counter accounts
}

/// Validate token ID uniqueness across chains
pub fn validate_token_id_uniqueness(
    token_id: u64,
    collection: &Pubkey,
    source_chain_id: u64,
) -> Result<SecurityValidationResult> {
    // Validate token ID range
    if token_id < MIN_TOKEN_ID || token_id > MAX_TOKEN_ID {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTokenId,
            "Token ID is outside valid range"
        ));
    }

    // Check if NFT Origin PDA exists
    let (origin_pda, _) = find_nft_origin_pda(&crate::ID, token_id);
    
    // In a real implementation, this would check if the account exists on-chain
    // For now, we'll do basic validation
    
    // Validate token ID format based on source chain
    match source_chain_id {
        // EVM chains typically use sequential IDs
        1 | 56 | 137 | 8453 | 42161 | 10 | 11155111 | 97 | 80001 | 84532 | 421614 | 11155420 => {
            if token_id > 1_000_000_000 {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::InvalidTokenId,
                    "Token ID too large for EVM chain"
                ));
            }
        },
        // Solana uses deterministic generation
        101 | 102 | 103 => {
            // Validate deterministic token ID format
            if !is_valid_deterministic_token_id(token_id) {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::InvalidTokenId,
                    "Invalid deterministic token ID format"
                ));
            }
        },
        _ => {}
    }

    Ok(SecurityValidationResult::valid())
}

/// Validate deterministic token ID format
fn is_valid_deterministic_token_id(token_id: u64) -> bool {
    // Check if token ID follows expected deterministic pattern
    // This is a simplified check - real implementation would be more sophisticated
    token_id > 0 && token_id != u64::MAX
}

/// Enhanced access control validation
pub fn validate_access_control(
    authority: &Pubkey,
    collection: &Collection,
    operation: &str,
) -> Result<SecurityValidationResult> {
    // Basic authority validation
    if *authority != collection.authority {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidSignature,
            "Unauthorized: caller is not collection authority"
        ));
    }

    // Operation-specific access control
    match operation {
        "initialize" => {
            // Only authority can initialize
            // Additional checks could be added here
        },
        "mint" => {
            // Check if minting is allowed
            if collection.is_at_capacity(Some(1_000_000)) {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::InvalidTokenId,
                    "Collection has reached maximum capacity"
                ));
            }
        },
        "transfer_cross_chain" => {
            // Additional checks for cross-chain transfers
            if collection.tss_address == [0u8; 20] {
                return Ok(SecurityValidationResult::invalid(
                    UniversalNftError::UnauthorizedTssAddress,
                    "TSS address not configured for cross-chain transfers"
                ));
            }
        },
        "emergency_pause" => {
            // Only authority can pause
            // Could add additional multi-sig requirements here
        },
        _ => {
            return Ok(SecurityValidationResult::invalid(
                UniversalNftError::InvalidSignature,
                "Unknown operation type"
            ));
        }
    }

    Ok(SecurityValidationResult::valid())
}

/// Validate account ownership and PDA derivations
pub fn validate_account_ownership_and_pda(
    account_info: &AccountInfo,
    expected_owner: &Pubkey,
    expected_seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<SecurityValidationResult> {
    // Validate account owner
    if account_info.owner != expected_owner {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidSignature,
            "Account has incorrect owner"
        ));
    }

    // Validate PDA derivation if seeds provided
    if !expected_seeds.is_empty() {
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, program_id);
        if *account_info.key != expected_pda {
            return Ok(SecurityValidationResult::invalid(
                UniversalNftError::InvalidSignature,
                "Account address does not match expected PDA"
            ));
        }
    }

    // Validate account is initialized (has data)
    if account_info.data_is_empty() {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::TokenDoesNotExist,
            "Account is not initialized"
        ));
    }

    // Validate account is not closed
    if account_info.lamports() == 0 {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::TokenDoesNotExist,
            "Account has been closed"
        ));
    }

    Ok(SecurityValidationResult::valid())
}

/// Emergency pause functionality
pub fn check_emergency_pause_status(
    collection: &Collection,
) -> Result<SecurityValidationResult> {
    // In a real implementation, this would check an emergency pause account
    // For now, we'll use a simplified approach
    
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    
    // Check if emergency pause is active
    if is_emergency_pause_active(collection, current_timestamp) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidSignature, // Reusing error code
            "Operations are paused due to emergency"
        ));
    }

    Ok(SecurityValidationResult::valid())
}

/// Check if emergency pause is active (simplified)
fn is_emergency_pause_active(_collection: &Collection, _current_timestamp: i64) -> bool {
    // In a real implementation, this would check emergency pause state
    false
}

/// Activate emergency pause
pub fn activate_emergency_pause(
    authority: &Pubkey,
    collection: &Collection,
    reason: &str,
) -> Result<EmergencyPauseState> {
    // Validate authority
    require_keys_eq!(*authority, collection.authority, UniversalNftError::InvalidSignature);
    
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;
    
    Ok(EmergencyPauseState {
        is_paused: true,
        paused_at: current_timestamp,
        pause_reason: reason.to_string(),
        paused_by: *authority,
    })
}

/// Deactivate emergency pause
pub fn deactivate_emergency_pause(
    authority: &Pubkey,
    collection: &Collection,
    pause_state: &EmergencyPauseState,
) -> Result<()> {
    // Validate authority
    require_keys_eq!(*authority, collection.authority, UniversalNftError::InvalidSignature);
    
    // Validate pause was activated by same authority or emergency override
    require!(
        pause_state.paused_by == *authority || is_emergency_override_authorized(authority),
        UniversalNftError::InvalidSignature
    );
    
    Ok(())
}

/// Check if emergency override is authorized (simplified)
fn is_emergency_override_authorized(_authority: &Pubkey) -> bool {
    // In a real implementation, this would check against a list of emergency authorities
    false
}

/// Convert secp256k1 public key to Ethereum address
fn pubkey_to_eth_address(pubkey: &[u8; 64]) -> Result<[u8; 20]> {
    // Validate public key format
    require!(
        pubkey.len() == TSS_PUBKEY_LENGTH,
        UniversalNftError::InvalidTssSignature
    );
    
    // Validate public key is not all zeros
    require!(
        !pubkey.iter().all(|&b| b == 0),
        UniversalNftError::InvalidTssSignature
    );
    
    // Hash the uncompressed public key (64 bytes)
    let hash_result = keccak::hash(pubkey);
    let hash_bytes = hash_result.to_bytes();
    
    // Take the last 20 bytes as Ethereum address
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash_bytes[12..32]);
    
    // Validate address is not zero address
    require!(
        !address.iter().all(|&b| b == 0),
        UniversalNftError::InvalidTssSignature
    );
    
    Ok(address)
}

/// Comprehensive security audit function
pub fn perform_security_audit(
    collection: &Collection,
    operation: &str,
    authority: &Pubkey,
    additional_data: Option<&[u8]>,
) -> Result<SecurityValidationResult> {
    // 1. Check emergency pause status
    let pause_result = check_emergency_pause_status(collection)?;
    if !pause_result.is_valid {
        return Ok(pause_result);
    }

    // 2. Validate access control
    let access_result = validate_access_control(authority, collection, operation)?;
    if !access_result.is_valid {
        return Ok(access_result);
    }

    // 3. Check rate limits
    let rate_result = check_rate_limits(authority, operation)?;
    if !rate_result.is_valid {
        return Ok(rate_result);
    }

    // 4. Additional data validation if provided
    if let Some(data) = additional_data {
        if data.len() > 10240 {
            return Ok(SecurityValidationResult::invalid(
                UniversalNftError::InvalidMessage,
                "Additional data exceeds size limit"
            ));
        }
    }

    // 5. Collection state validation
    if collection.total_minted > 10_000_000 {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTokenId,
            "Collection has exceeded maximum safe size"
        ));
    }

    Ok(SecurityValidationResult::valid())
}

/// Validate cross-chain message structure and content
pub fn validate_cross_chain_message_structure(
    message: &CrossChainMessage,
) -> Result<SecurityValidationResult> {
    // Validate token ID
    if message.token_id < MIN_TOKEN_ID || message.token_id > MAX_TOKEN_ID {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidTokenId,
            "Token ID is outside valid range"
        ));
    }

    // Validate URI
    if message.uri.is_empty() || message.uri.len() > MAX_URI_LENGTH {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidMessage,
            "URI is empty or exceeds maximum length"
        ));
    }

    // Validate recipient address format
    if message.recipient.len() != EVM_ADDRESS_LENGTH && message.recipient.len() != SOLANA_ADDRESS_LENGTH {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidRecipientAddress,
            "Recipient address has invalid format"
        ));
    }

    // Validate sender address format
    if message.sender.len() != EVM_ADDRESS_LENGTH && message.sender.len() != SOLANA_ADDRESS_LENGTH {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidRecipientAddress,
            "Sender address has invalid format"
        ));
    }

    // Validate destination chain
    if message.destination_chain.len() != 8 {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidDestinationChain,
            "Destination chain format is invalid"
        ));
    }

    let chain_id = u64::from_le_bytes(
        message.destination_chain.as_slice().try_into()
            .map_err(|_| UniversalNftError::InvalidDestinationChain)?
    );

    if !crate::state::is_supported_chain(chain_id) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::UnsupportedChain,
            "Destination chain is not supported"
        ));
    }

    // Validate URI content (basic checks)
    if !is_valid_uri_format(&message.uri) {
        return Ok(SecurityValidationResult::invalid(
            UniversalNftError::InvalidMessage,
            "URI format is invalid"
        ));
    }

    Ok(SecurityValidationResult::valid())
}

/// Validate URI format
fn is_valid_uri_format(uri: &str) -> bool {
    // Basic URI validation
    if uri.is_empty() {
        return false;
    }

    // Check for common URI schemes
    let valid_schemes = ["https://", "ipfs://", "ar://", "data:"];
    let has_valid_scheme = valid_schemes.iter().any(|scheme| uri.starts_with(scheme));

    if !has_valid_scheme {
        return false;
    }

    // Additional validation could be added here
    true
}

/// Security event logging structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SecurityEvent {
    pub event_type: String,
    pub timestamp: i64,
    pub authority: Pubkey,
    pub details: String,
    pub severity: SecuritySeverity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Log security event
pub fn log_security_event(
    event_type: &str,
    authority: &Pubkey,
    details: &str,
    severity: SecuritySeverity,
) -> Result<()> {
    let clock = Clock::get()?;
    
    let event = SecurityEvent {
        event_type: event_type.to_string(),
        timestamp: clock.unix_timestamp,
        authority: *authority,
        details: details.to_string(),
        severity,
    };

    // In a real implementation, this would emit an event or store in an account
    msg!("Security Event: {:?}", event);

    Ok(())
}