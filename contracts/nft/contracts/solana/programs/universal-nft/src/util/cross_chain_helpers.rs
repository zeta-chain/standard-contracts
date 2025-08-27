use anchor_lang::prelude::*;
use crate::error::UniversalNftError;

/// Cross-chain message data parsing and utilities
/// This module contains helper functions for encoding/decoding cross-chain NFT data

/// Extract a length-prefixed string from byte array
/// Format: length(4 bytes, big-endian) + string_data
fn extract_length_prefixed_string(data: &[u8], cursor: usize) -> Result<(String, usize)> {
    if cursor + 4 > data.len() {
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    
    // Read length (4 bytes, big-endian)
    let len = u32::from_be_bytes([
        data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3],
    ]) as usize;
    
    let new_cursor = cursor + 4;
    if new_cursor + len > data.len() {
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    
    // Extract string data
    let string_data = &data[new_cursor..new_cursor + len];
    let string_value = String::from_utf8(string_data.to_vec())
        .map_err(|_| UniversalNftError::InvalidDataFormat)?;
    
    Ok((string_value, new_cursor + len))
}

/// Encode a length-prefixed string into byte array
/// Format: length(4 bytes, big-endian) + string_data
fn encode_length_prefixed_string(data: &mut Vec<u8>, string: &str) {
    let string_bytes = string.as_bytes();
    let len = string_bytes.len() as u32;
    
    // Write length (4 bytes, big-endian)
    data.extend_from_slice(&len.to_be_bytes());
    
    // Write string data
    data.extend_from_slice(string_bytes);
}

/// Convert hex string address to 20-byte array (for Ethereum-style addresses)
/// Supports both "0x" prefixed and raw hex strings
pub fn hex_string_to_address(address_str: &str) -> Result<[u8; 20]> {
    let hex_str = if address_str.starts_with("0x") {
        &address_str[2..]
    } else {
        address_str
    };
    
    if hex_str.len() != 40 {
        return Err(UniversalNftError::InvalidRecipientAddress.into());
    }
    
    let mut bytes = [0u8; 20];
    for i in 0..20 {
        let byte_str = &hex_str[i*2..i*2+2];
        bytes[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|_| UniversalNftError::InvalidRecipientAddress)?;
    }
    
    Ok(bytes)
}

/// Convert 20-byte address to hex string with "0x" prefix
pub fn address_to_hex_string(address: &[u8; 20]) -> String {
    format!("0x{}", hex::encode(address))
}
/// Decode an EVM ABI-encoded NFT message used by ZEVM Universal contracts when forwarding to Solana.
/// Tuple layout (Solidity): (address receiver, uint256 tokenId, string uri, uint256 amount, address sender)
/// ABI encoding:
/// - Head (5 x 32 bytes):
///   [0] receiver: 12 zero bytes + 20 byte address
///   [1] tokenId: 32-byte big-endian uint256
///   [2] offset to uri (uint256, from start of payload)
///   [3] amount: 32-byte big-endian uint256 (can be ignored; `amount` also provided by gateway)
///   [4] sender: 12 zero bytes + 20 byte address
/// - Tail at offset: uri length (uint256) + uri bytes + padding to 32 bytes
pub fn decode_evm_abi_nft_message(data: &[u8]) -> Result<([u8; 32], [u8; 20], String, [u8; 20])> {
    // Need at least 5 words for head and one word for string length
    if data.len() < 32 * 6 {
        return Err(UniversalNftError::InvalidDataFormat.into());
    }

    // Helper to extract a 32-byte word at index
    let word = |i: usize| -> &[u8] { &data[i * 32..(i + 1) * 32] };

    // receiver (last 20 bytes of word 0)
    let mut receiver = [0u8; 20];
    receiver.copy_from_slice(&word(0)[12..32]);

    // tokenId (word 1)
    let mut token_id = [0u8; 32];
    token_id.copy_from_slice(word(1));

    // uri offset (word 2) as big-endian; must fit in usize and high 24 bytes zero
    let off_word = word(2);
    if off_word[..24].iter().any(|&b| b != 0) {
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    let mut off_u64_bytes = [0u8; 8];
    off_u64_bytes.copy_from_slice(&off_word[24..32]);
    let offset = u64::from_be_bytes(off_u64_bytes) as usize;
    if offset + 32 > data.len() { // must have length word
        return Err(UniversalNftError::InvalidDataFormat.into());
    }

    // sender (last 20 bytes of word 4)
    let mut sender = [0u8; 20];
    sender.copy_from_slice(&word(4)[12..32]);

    // Tail: string length (u256 big-endian, must fit in usize)
    let len_word = &data[offset..offset + 32];
    if len_word[..24].iter().any(|&b| b != 0) {
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    let mut len_u64_bytes = [0u8; 8];
    len_u64_bytes.copy_from_slice(&len_word[24..32]);
    let uri_len = u64::from_be_bytes(len_u64_bytes) as usize;

    let start = offset + 32;
    if start + uri_len > data.len() {
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    let uri_bytes = &data[start..start + uri_len];
    let uri = String::from_utf8(uri_bytes.to_vec())
        .map_err(|_| UniversalNftError::InvalidDataFormat)?;

    Ok((token_id, receiver, uri, sender))
}