use anchor_lang::prelude::*;
use crate::errors::Errors;

pub fn parse_hex_address_to_bytes(address_str: &str) -> Result<[u8; 20]> {
    let hex_str = if address_str.starts_with("0x") {
        &address_str[2..]
    } else {
        address_str
    };
    
    if hex_str.len() != 40 {
        return Err(Errors::InvalidRecipientAddress.into());
    }
    
    let mut bytes = [0u8; 20];
    for i in 0..20 {
        let byte_str = &hex_str[i*2..i*2+2];
        bytes[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|_| Errors::InvalidRecipientAddress)?;
    }
    
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_address_to_bytes() {
        let valid_address = "0x1234567890123456789012345678901234567890";
        let result = parse_hex_address_to_bytes(valid_address).unwrap();
        assert_eq!(result, [0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90]);

        let invalid_length_address = "0x123456789012345678901234567890123456789"; // 39 chars
        assert!(parse_hex_address_to_bytes(invalid_length_address).is_err());

        let invalid_hex_address = "0x123456789012345678901234567890123456789g"; // contains 'g'
        assert!(parse_hex_address_to_bytes(invalid_hex_address).is_err());
    }
}
