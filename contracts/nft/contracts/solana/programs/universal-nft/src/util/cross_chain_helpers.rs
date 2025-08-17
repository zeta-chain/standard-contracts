use anchor_lang::prelude::*;
use crate::error::UniversalNftError;

/// Cross-chain message data parsing and utilities
/// This module contains helper functions for encoding/decoding cross-chain NFT data

/// Decode NFT data from a cross-chain message
/// Parses the message format: token_id(32) + source_chain(8) + metadata_uri + name + symbol
/// Each string is length-prefixed with a 4-byte big-endian length
pub fn decode_nft_data(data: &[u8]) -> Result<([u8; 32], u64, String, String, String)> {
    if data.len() < 40 { // Minimum size: 32 (token_id) + 8 (source_chain)
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    
    let mut cursor = 0;
    
    // Extract token_id (32 bytes)
    let mut token_id = [0u8; 32];
    token_id.copy_from_slice(&data[cursor..cursor + 32]);
    cursor += 32;
    
    // Extract source_chain (8 bytes, big-endian)
    let source_chain = u64::from_be_bytes([
        data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3],
        data[cursor + 4], data[cursor + 5], data[cursor + 6], data[cursor + 7],
    ]);
    cursor += 8;
    
    // Extract strings (each is length-prefixed)
    let (metadata_uri, new_cursor) = extract_length_prefixed_string(data, cursor)?;
    cursor = new_cursor;
    
    let (name, new_cursor) = extract_length_prefixed_string(data, cursor)?;
    cursor = new_cursor;
    
    let (symbol, _) = extract_length_prefixed_string(data, cursor)?;
    
    Ok((token_id, source_chain, metadata_uri, name, symbol))
}

/// Decode NFT data from a cross-chain message with recipient
/// Parses the message format: token_id(32) + source_chain(8) + recipient(32) + metadata_uri + name + symbol
/// Each string is length-prefixed with a 4-byte big-endian length
pub fn decode_nft_data_with_recipient(data: &[u8]) -> Result<([u8; 32], u64, [u8; 32], String, String, String)> {
    if data.len() < 72 { // Minimum size: 32 (token_id) + 8 (source_chain) + 32 (recipient)
        return Err(UniversalNftError::InvalidDataFormat.into());
    }
    
    let mut cursor = 0;
    
    // Extract token_id (32 bytes)
    let mut token_id = [0u8; 32];
    token_id.copy_from_slice(&data[cursor..cursor + 32]);
    cursor += 32;
    
    // Extract source_chain (8 bytes, big-endian)
    let source_chain = u64::from_be_bytes([
        data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3],
        data[cursor + 4], data[cursor + 5], data[cursor + 6], data[cursor + 7],
    ]);
    cursor += 8;
    
    // Extract recipient (32 bytes)
    let mut recipient_bytes = [0u8; 32];
    recipient_bytes.copy_from_slice(&data[cursor..cursor + 32]);
    cursor += 32;
    
    // Extract strings (each is length-prefixed)
    let (metadata_uri, new_cursor) = extract_length_prefixed_string(data, cursor)?;
    cursor = new_cursor;
    
    let (name, new_cursor) = extract_length_prefixed_string(data, cursor)?;
    cursor = new_cursor;
    
    let (symbol, _) = extract_length_prefixed_string(data, cursor)?;
    
    Ok((token_id, source_chain, recipient_bytes, metadata_uri, name, symbol))
}

/// Encode NFT data for cross-chain message
/// Creates the message format: token_id(32) + source_chain(8) + metadata_uri + name + symbol
pub fn encode_nft_data(
    token_id: [u8; 32],
    source_chain: u64,
    metadata_uri: &str,
    name: &str,
    symbol: &str,
) -> Vec<u8> {
    let mut data = Vec::new();
    
    // Token ID (32 bytes)
    data.extend_from_slice(&token_id);
    
    // Source chain (8 bytes, big-endian)
    data.extend_from_slice(&source_chain.to_be_bytes());
    
    // Strings (each length-prefixed with 4-byte big-endian length)
    encode_length_prefixed_string(&mut data, metadata_uri);
    encode_length_prefixed_string(&mut data, name);
    encode_length_prefixed_string(&mut data, symbol);
    
    data
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let token_id = [1u8; 32];
        let source_chain = 1337u64;
        let metadata_uri = "https://gist.githubusercontent.com/adamliu84/d84f1058ea5d45d6fc03472a9fa2dbb2/raw/cf154d08ed197aef4ee157055e435e999e23f0e8/0.json";
        let name = "Test NFT";
        let symbol = "TEST";
        
        let encoded = encode_nft_data(token_id, source_chain, metadata_uri, name, symbol);
        let (decoded_token_id, decoded_source_chain, decoded_uri, decoded_name, decoded_symbol) = 
            decode_nft_data(&encoded).unwrap();
        
        assert_eq!(decoded_token_id, token_id);
        assert_eq!(decoded_source_chain, source_chain);
        assert_eq!(decoded_uri, metadata_uri);
        assert_eq!(decoded_name, name);
        assert_eq!(decoded_symbol, symbol);
    }
}
