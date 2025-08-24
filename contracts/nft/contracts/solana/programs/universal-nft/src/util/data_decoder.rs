use anchor_lang::prelude::*;
use crate::errors::Errors;

pub struct CrossChainDataDecoder;

impl CrossChainDataDecoder {
    pub fn decode_nft_data(data: &[u8]) -> Result<([u8; 32], u64, [u8; 20], String, String, String)> {
        if data.len() < 60 { // minimum size for fixed fields
            return Err(Errors::InvalidDataFormat.into());
        }

        let mut offset = 0;

        let token_id = data[offset..offset + 32].try_into()
            .map_err(|_| Errors::InvalidDataFormat)?;
        offset += 32;

        let origin_chain_bytes = data[offset..offset + 8].try_into()
            .map_err(|_| Errors::InvalidDataFormat)?;
        let origin_chain = u64::from_le_bytes(origin_chain_bytes);
        offset += 8;

        let recipient = data[offset..offset + 20].try_into()
            .map_err(|_| Errors::InvalidDataFormat)?;
        offset += 20;

        let (uri, offset) = Self::decode_string(data, offset)?;
        let (name, offset) = Self::decode_string(data, offset)?;
        let (symbol, _) = Self::decode_string(data, offset)?;

        Ok((token_id, origin_chain, recipient, uri, name, symbol))
    }

    fn decode_string(data: &[u8], mut offset: usize) -> Result<(String, usize)> {
        if offset + 4 > data.len() {
            return Err(Errors::InvalidDataFormat.into());
        }

        let length_bytes = data[offset..offset + 4].try_into()
            .map_err(|_| Errors::InvalidDataFormat)?;
        let length = u32::from_le_bytes(length_bytes) as usize;
        offset += 4;

        if offset + length > data.len() {
            return Err(Errors::InvalidDataFormat.into());
        }

        let string_data = &data[offset..offset + length];
        let string = String::from_utf8(string_data.to_vec())
            .map_err(|_| Errors::InvalidDataFormat)?;

        Ok((string, offset + length))
    }

    pub fn validate_decoded_data(
        name: &str,
        symbol: &str,
        uri: &str,
    ) -> Result<()> {
        if name.len() > 32 {
            return Err(Errors::NameTooLong.into());
        }

        if symbol.len() > 10 {
            return Err(Errors::SymbolTooLong.into());
        }

        if uri.len() > 200 {
            return Err(Errors::UriTooLong.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_nft_data() {
        let token_id = [1u8; 32];
        let origin_chain = 1u64;
        let recipient = [2u8; 20];
        let uri = "https://raw.githubusercontent.com/metaplex-foundation/gem-farm/ff5535c73cdd52148c2db60a626a55059ecad9d1/tests/artifacts/testMetadata.json".to_string();
        let name = "Test NFT".to_string();
        let symbol = "TNFT".to_string();

        let mut data = Vec::new();
        data.extend_from_slice(&token_id);
        data.extend_from_slice(&origin_chain.to_le_bytes());
        data.extend_from_slice(&recipient);
        
        data.extend_from_slice(&(uri.len() as u32).to_le_bytes());
        data.extend_from_slice(uri.as_bytes());
        
        data.extend_from_slice(&(name.len() as u32).to_le_bytes());
        data.extend_from_slice(name.as_bytes());
        
        data.extend_from_slice(&(symbol.len() as u32).to_le_bytes());
        data.extend_from_slice(symbol.as_bytes());

        let result = CrossChainDataDecoder::decode_nft_data(&data).unwrap();
        assert_eq!(result.0, token_id);
        assert_eq!(result.1, origin_chain);
        assert_eq!(result.2, recipient);
        assert_eq!(result.3, uri);
        assert_eq!(result.4, name);
        assert_eq!(result.5, symbol);
    }

    #[test]
    fn test_decode_invalid_data() {
        let invalid_data = [1u8; 30];
        assert!(CrossChainDataDecoder::decode_nft_data(&invalid_data).is_err());
    }
}
