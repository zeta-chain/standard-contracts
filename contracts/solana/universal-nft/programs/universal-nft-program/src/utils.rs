use anchor_lang::prelude::*;
use solana_program::{
    clock::Clock,
    keccak::{hash as keccak_hash, Hash as KeccakHash},
    secp256k1_recover::{secp256k1_recover, Secp256k1Pubkey},
};
use crate::{constants::*, errors::*, state::*};
use sha2::{Digest, Sha256};

pub fn generate_unique_token_id(mint: &Pubkey, clock: &Clock) -> Result<u64> {
    let mut hasher = Sha256::new();
    hasher.update(mint.as_ref());
    hasher.update(clock.unix_timestamp.to_le_bytes());
    hasher.update(clock.slot.to_le_bytes());
    
    let hash_result = hasher.finalize();
    let token_id = u64::from_le_bytes([
        hash_result[0],
        hash_result[1],
        hash_result[2],
        hash_result[3],
        hash_result[4],
        hash_result[5],
        hash_result[6],
        hash_result[7],
    ]);
    
    Ok(token_id)
}

pub fn verify_tss_signature(
    message: &[u8],
    signature: &[u8; 64],
    recovery_id: u8,
    expected_tss_address: &[u8; 20],
) -> Result<()> {
    // Create Ethereum-style message hash (EIP-191)
    let message_hash = create_ethereum_signed_message_hash(message);
    
    let recovered_pubkey = secp256k1_recover(&message_hash, recovery_id, signature)
        .map_err(|_| UniversalNftError::InvalidSignatureRecovery)?;

    let eth_address = pubkey_to_eth_address(&recovered_pubkey);
    
    require!(
        eth_address == *expected_tss_address,
        UniversalNftError::InvalidSignature
    );
    
    Ok(())
}

/// Create Ethereum-style signed message hash according to EIP-191
pub fn create_ethereum_signed_message_hash(message: &[u8]) -> [u8; 32] {
    let prefix = b"\x19Ethereum Signed Message:\n";
    let message_len = message.len().to_string();
    
    let mut full_message = Vec::new();
    full_message.extend_from_slice(prefix);
    full_message.extend_from_slice(message_len.as_bytes());
    full_message.extend_from_slice(message);
    
    let hash = keccak_hash(&full_message);
    hash.to_bytes()
}

/// Verify a message was signed by ZetaChain TSS with proper message formatting
pub fn verify_cross_chain_message(
    chain_id: u64,
    nonce: u64,
    message_type: &CrossChainMessageType,
    signature: &[u8; 64],
    recovery_id: u8,
    expected_tss_address: &[u8; 20],
) -> Result<()> {
    // Create the message hash that should have been signed
    let message_hash = create_cross_chain_message_hash(chain_id, nonce, message_type)?;
    
    // Create domain-separated message for signature
    let domain_message = {
        let mut msg = Vec::new();
        msg.extend_from_slice(b"ZETACHAIN_CROSS_CHAIN:");
        msg.extend_from_slice(&chain_id.to_be_bytes());
        msg.extend_from_slice(&nonce.to_be_bytes());
        msg.extend_from_slice(&message_hash);
        msg
    };
    
    verify_tss_signature(&domain_message, signature, recovery_id, expected_tss_address)
}

pub fn pubkey_to_eth_address(pubkey: &Secp256k1Pubkey) -> [u8; 20] {
    let pubkey_bytes = pubkey.to_bytes();
    
    let uncompressed_pubkey = if pubkey_bytes[0] == 0x04 {
        &pubkey_bytes[1..]
    } else {
        &pubkey_bytes[..]
    };
    
    let hash = keccak_hash(uncompressed_pubkey);
    let mut eth_address = [0u8; 20];
    eth_address.copy_from_slice(&hash.to_bytes()[12..]);
    eth_address
}

pub fn create_cross_chain_message_hash(
    chain_id: u64,
    nonce: u64,
    message_type: &CrossChainMessageType,
) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    
    hasher.update(CROSS_CHAIN_MESSAGE_VERSION.to_le_bytes());
    hasher.update(chain_id.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    
    let message_bytes = match message_type {
        CrossChainMessageType::MintRequest { recipient, metadata } => {
            let mut msg = Vec::new();
            msg.extend_from_slice(b"MINT_REQUEST");
            msg.extend_from_slice(recipient.as_ref());
            msg.extend_from_slice(&metadata.try_to_vec()?);
            msg
        }
        CrossChainMessageType::BurnConfirmation { token_id, burned_amount } => {
            let mut msg = Vec::new();
            msg.extend_from_slice(b"BURN_CONFIRMATION");
            msg.extend_from_slice(&token_id.to_le_bytes());
            msg.extend_from_slice(&burned_amount.to_le_bytes());
            msg
        }
        CrossChainMessageType::RevertRequest { original_transaction, revert_context } => {
            let mut msg = Vec::new();
            msg.extend_from_slice(b"REVERT_REQUEST");
            msg.extend_from_slice(original_transaction);
            msg.extend_from_slice(&revert_context.try_to_vec()?);
            msg
        }
    };
    
    hasher.update(&message_bytes);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Ok(hash)
}

pub fn validate_metadata(metadata: &CrossChainNftMetadata) -> Result<()> {
    require!(
        metadata.name.len() <= MAX_NAME_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        metadata.symbol.len() <= MAX_SYMBOL_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        metadata.uri.len() <= MAX_URI_LENGTH,
        UniversalNftError::MetadataTooLong
    );
    
    require!(
        metadata.attributes.len() <= MAX_ATTRIBUTES_COUNT,
        UniversalNftError::AttributesLimitExceeded
    );
    
    for attribute in &metadata.attributes {
        require!(
            attribute.trait_type.len() <= MAX_ATTRIBUTE_NAME_LENGTH,
            UniversalNftError::InvalidAttributeData
        );
        require!(
            attribute.value.len() <= MAX_ATTRIBUTE_VALUE_LENGTH,
            UniversalNftError::InvalidAttributeData
        );
    }
    
    Ok(())
}

pub fn validate_destination_address(address: &[u8]) -> Result<()> {
    require!(
        address.len() <= MAX_DESTINATION_ADDRESS_LENGTH,
        UniversalNftError::InvalidDestinationAddress
    );
    
    require!(
        !address.is_empty(),
        UniversalNftError::InvalidDestinationAddress
    );
    
    Ok(())
}

pub fn calculate_metadata_hash(metadata: &CrossChainNftMetadata) -> Result<[u8; 32]> {
    let metadata_bytes = metadata.try_to_vec()?;
    let mut hasher = Sha256::new();
    hasher.update(&metadata_bytes);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Ok(hash)
}

pub fn generate_collection_seeds(authority: &Pubkey) -> Vec<Vec<u8>> {
    vec![
        COLLECTION_SEED.to_vec(),
        authority.as_ref().to_vec(),
    ]
}

pub fn generate_nft_state_seeds(mint: &Pubkey) -> Vec<Vec<u8>> {
    vec![
        NFT_STATE_SEED.to_vec(),
        mint.as_ref().to_vec(),
    ]
}

pub fn generate_gateway_message_seeds(sender: &[u8; 20], nonce: u64) -> Vec<Vec<u8>> {
    vec![
        GATEWAY_MESSAGE_SEED.to_vec(),
        sender.to_vec(),
        nonce.to_le_bytes().to_vec(),
    ]
}

pub fn safe_add_u64(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| UniversalNftError::MathematicalOverflow.into())
}

pub fn safe_multiply_u64(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b)
        .ok_or_else(|| UniversalNftError::MathematicalOverflow.into())
}