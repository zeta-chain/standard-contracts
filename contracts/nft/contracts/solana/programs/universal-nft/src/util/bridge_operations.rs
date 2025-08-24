/// Encode the Anchor instruction data for the Cross-Chain Bridge `transfer_and_invoke` method.
/// Layout per Anchor: [discriminator(8)] + amount(u64 LE) + receiver([u8;20]) + message(Vec<u8>) + revert_options(Option<RevertOptions>)
pub fn encode_bridge_transfer_and_invoke_instruction_data(amount_lamports: u64, receiver: [u8; 20], message: &[u8]) -> Vec<u8> {
    // Discriminator = sha256("global:transfer_and_invoke")[..8]
    let disc = anchor_lang::solana_program::hash::hash(b"global:transfer_and_invoke").to_bytes();
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

/// Encode message expected by UniversalAssetCore.onCall on ZEVM:
/// (address receiver, uint256 assetId, string uri, uint256 gasAmount, address sender)
/// Notes:
/// - Addresses occupy 32 bytes (12 zero prefix + 20 bytes)
/// - assetId and gasAmount are 32-byte big-endian words
/// - string is dynamic: head holds offset; tail holds len + data + padding
pub fn encode_evm_oncall_message(
    receiver: [u8; 20],
    asset_id_be32: [u8; 32],
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
    // 2) assetId (32 bytes, already big-endian)
    out.extend_from_slice(&asset_id_be32);
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
    fn test_encode_bridge_transfer_and_invoke_ix_data_layout() {
        let receiver = [0x22u8; 20];
        let message = vec![0x01, 0x02];
        let amt = 2_000_000u64;
        let data = encode_bridge_transfer_and_invoke_instruction_data(amt, receiver, &message);
        assert_eq!(&data[..8], &anchor_lang::solana_program::hash::hash(b"global:transfer_and_invoke").to_bytes()[..8]);
        let amt_le = u64::from_le_bytes(data[8..16].try_into().unwrap());
        assert_eq!(amt_le, amt);
        assert_eq!(&data[16..36], &receiver);
        let len_le = u32::from_le_bytes(data[36..40].try_into().unwrap());
        assert_eq!(len_le as usize, message.len());
        assert_eq!(&data[40..42], &message[..]);
        assert_eq!(data[42], 0u8); // None
    }
}
