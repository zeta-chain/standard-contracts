pub fn encode_bridge_deposit_and_call_instruction_data(amount_lamports: u64, receiver: [u8; 20], message: &[u8]) -> Vec<u8> {
    // Discriminator = sha256("global:deposit_and_call")[..8]
    let disc = anchor_lang::solana_program::hash::hash(b"global:deposit_and_call").to_bytes();
    let mut data = Vec::with_capacity(8 + 8 + 20 + 4 + message.len() + 1);
    data.extend_from_slice(&disc[..8]);
    data.extend_from_slice(&amount_lamports.to_le_bytes());
    data.extend_from_slice(&receiver);
    data.extend_from_slice(&(message.len() as u32).to_le_bytes());
    data.extend_from_slice(message);
    data.push(0u8);
    data
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_bridge_deposit_and_call_ix_data_layout() {
        let receiver = [0x22u8; 20];
        let message = vec![0x01, 0x02];
        let amt = 2_000_000u64;
        let data = encode_bridge_deposit_and_call_instruction_data(amt, receiver, &message);
        assert_eq!(&data[..8], &anchor_lang::solana_program::hash::hash(b"global:deposit_and_call").to_bytes()[..8]);
        let amt_le = u64::from_le_bytes(data[8..16].try_into().unwrap());
        assert_eq!(amt_le, amt);
        assert_eq!(&data[16..36], &receiver);
        let len_le = u32::from_le_bytes(data[36..40].try_into().unwrap());
        assert_eq!(len_le as usize, message.len());
        assert_eq!(&data[40..42], &message[..]);
        assert_eq!(data[42], 0u8);
    }
}
