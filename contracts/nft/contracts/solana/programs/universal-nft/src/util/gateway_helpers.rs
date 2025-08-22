use anchor_lang::prelude::*;

/// Encode the Anchor instruction data for the ZetaChain Gateway `call` method.
/// Layout per Anchor: [discriminator(8)] + receiver([u8;20]) + message(Vec<u8>) + revert_options(Option<RevertOptions>)
/// - receiver: fixed 20 bytes
/// - message: u32 LE length + bytes
/// - revert_options: None encoded as 0u8
pub fn encode_gateway_call_ix_data(receiver: [u8; 20], message: &[u8]) -> Vec<u8> {
    // Discriminator = sha256("global:call")[..8]
    let disc = anchor_lang::solana_program::hash::hash(b"global:call").to_bytes();
    let mut data = Vec::with_capacity(8 + 20 + 4 + message.len() + 1);
    data.extend_from_slice(&disc[..8]);
    data.extend_from_slice(&receiver);
    data.extend_from_slice(&(message.len() as u32).to_le_bytes());
    data.extend_from_slice(message);
    // Option::None
    data.push(0u8);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_gateway_call_ix_data_layout() {
        let receiver = [0x11u8; 20];
        let message = vec![0xAA, 0xBB, 0xCC];
        let data = encode_gateway_call_ix_data(receiver, &message);
        assert_eq!(&data[..8], &anchor_lang::solana_program::hash::hash(b"global:call").to_bytes()[..8]);
        assert_eq!(&data[8..28], &receiver);
        let len_le = u32::from_le_bytes(data[28..32].try_into().unwrap());
        assert_eq!(len_le as usize, message.len());
        assert_eq!(&data[32..35], &message[..]);
        assert_eq!(data[35], 0u8); // None
    }
}
