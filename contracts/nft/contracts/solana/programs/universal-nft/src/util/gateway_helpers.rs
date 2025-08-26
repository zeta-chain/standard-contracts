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

/// Encode the Anchor instruction data for the ZetaChain Gateway `deposit_and_call` method.
/// Layout per Anchor: [discriminator(8)] + amount(u64 LE) + receiver([u8;20]) + message(Vec<u8>) + revert_options(Option<RevertOptions>)
pub fn encode_gateway_deposit_and_call_ix_data(amount_lamports: u64, receiver: [u8; 20], message: &[u8]) -> Vec<u8> {
    // Discriminator = sha256("global:deposit_and_call")[..8]
    let disc = anchor_lang::solana_program::hash::hash(b"global:deposit_and_call").to_bytes();
    let mut data = Vec::with_capacity(8 + 8 + 20 + 4 + message.len() + 1);
    data.extend_from_slice(&disc[..8]);
    data.extend_from_slice(&amount_lamports.to_le_bytes());
    data.extend_from_slice(&receiver);
    data.extend_from_slice(&(message.len() as u32).to_le_bytes());
    data.extend_from_slice(message);
    // Option::None for revert_options
    data.push(0u8);
    data
}

/// Encode the EVM ABI tuple (address destination, address receiver, uint256 tokenId, string uri, address sender)
/// into bytes expected by a Solidity `abi.decode(message, (address,address,uint256,string,address))` call.
/// Notes:
/// - Addresses are right-padded within a 32-byte word (12 bytes leading zeros + 20 bytes address)
/// - uint256 is encoded as a 32-byte big-endian word; we treat the provided 32-byte token_id as big-endian
/// - string is encoded as length (32-byte) + bytes + 0-padding to 32-byte boundary
pub fn encode_evm_nft_message(
    destination: [u8; 20],
    receiver: [u8; 20],
    token_id_be32: [u8; 32],
    uri: &str,
    sender: [u8; 20],
) -> Vec<u8> {
    // Head = 5 * 32 bytes
    let head_len = 32 * 5;
    let uri_bytes = uri.as_bytes();
    let uri_len = uri_bytes.len();
    let uri_padded_len = ((uri_len + 31) / 32) * 32; // round up to 32

    let total_len = head_len + 32 /*len*/ + uri_padded_len;
    let mut out = Vec::with_capacity(total_len);

    // 1) destination address (32 bytes: 12 zero + 20 bytes)
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&destination);
    // 2) receiver address
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&receiver);
    // 3) tokenId (32 bytes big-endian). Provided as 32-bytes already.
    out.extend_from_slice(&token_id_be32);
    // 4) offset to string data (from start of payload)
    let offset = (32 * 5) as u64;
    let mut off_buf = [0u8; 32];
    // big-endian u256 representation of offset (fits in 64 bits)
    let off_bytes = offset.to_be_bytes();
    off_buf[32 - off_bytes.len()..].copy_from_slice(&off_bytes);
    out.extend_from_slice(&off_buf);
    // 5) sender address
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&sender);

    // Tail: string length (32 bytes) + data + padding
    let mut len_buf = [0u8; 32];
    let len_be = (uri_len as u64).to_be_bytes();
    len_buf[32 - len_be.len()..].copy_from_slice(&len_be);
    out.extend_from_slice(&len_buf);
    out.extend_from_slice(uri_bytes);
    // Pad to 32-byte boundary
    let pad_len = uri_padded_len - uri_len;
    if pad_len > 0 { out.extend_from_slice(&vec![0u8; pad_len]); }

    out
}

/// Encode message expected by UniversalNFTCore.onCall on ZEVM:
/// (address receiver, uint256 tokenId, string uri, uint256 gasAmount, address sender)
/// Notes:
/// - Addresses occupy 32 bytes (12 zero prefix + 20 bytes)
/// - tokenId and gasAmount are 32-byte big-endian words
/// - string is dynamic: head holds offset; tail holds len + data + padding
pub fn encode_evm_oncall_message(
    receiver: [u8; 20],
    token_id_be32: [u8; 32],
    uri: &str,
    gas_amount: u64,
    sender: [u8; 20],
) -> Vec<u8> {
    // Head = 5 * 32 bytes
    let head_len = 32 * 5;
    let uri_bytes = uri.as_bytes();
    let uri_len = uri_bytes.len();
    let uri_padded_len = ((uri_len + 31) / 32) * 32;

    let total_len = head_len + 32 /*len*/ + uri_padded_len;
    let mut out = Vec::with_capacity(total_len);

    // 1) receiver address
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&receiver);
    // 2) tokenId (32 bytes, already big-endian)
    out.extend_from_slice(&token_id_be32);
    // 3) offset to string (from start)
    let offset = (32 * 5) as u64;
    let mut off_buf = [0u8; 32];
    let off_bytes = offset.to_be_bytes();
    off_buf[32 - off_bytes.len()..].copy_from_slice(&off_bytes);
    out.extend_from_slice(&off_buf);
    // 4) gasAmount (u256 big-endian from u64)
    let mut gas_buf = [0u8; 32];
    let gas_be = gas_amount.to_be_bytes();
    gas_buf[32 - gas_be.len()..].copy_from_slice(&gas_be);
    out.extend_from_slice(&gas_buf);
    // 5) sender address
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&sender);

    // Tail: string (len + bytes + padding)
    let mut len_buf = [0u8; 32];
    let len_be = (uri_len as u64).to_be_bytes();
    len_buf[32 - len_be.len()..].copy_from_slice(&len_be);
    out.extend_from_slice(&len_buf);
    out.extend_from_slice(uri_bytes);
    let pad_len = uri_padded_len - uri_len;
    if pad_len > 0 {
        out.extend_from_slice(&vec![0u8; pad_len]);
    }

    out
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

    #[test]
    fn test_encode_gateway_deposit_and_call_ix_data_layout() {
        let receiver = [0x22u8; 20];
        let message = vec![0x01, 0x02];
        let amt = 2_000_000u64;
        let data = encode_gateway_deposit_and_call_ix_data(amt, receiver, &message);
        assert_eq!(&data[..8], &anchor_lang::solana_program::hash::hash(b"global:deposit_and_call").to_bytes()[..8]);
        let amt_le = u64::from_le_bytes(data[8..16].try_into().unwrap());
        assert_eq!(amt_le, amt);
        assert_eq!(&data[16..36], &receiver);
        let len_le = u32::from_le_bytes(data[36..40].try_into().unwrap());
        assert_eq!(len_le as usize, message.len());
        assert_eq!(&data[40..42], &message[..]);
        assert_eq!(data[42], 0u8); // None
    }
}
