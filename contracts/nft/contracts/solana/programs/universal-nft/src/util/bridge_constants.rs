use anchor_lang::solana_program::{pubkey, pubkey::Pubkey};

pub const MAX_RECIPIENT_ADDRESS_LENGTH: usize = 128;

// zetachain network id 1
// Ethereum network id 2
// BNB network id 3
// Solana network id 4
pub const SOLANA_NETWORK_ID: u64 = 4;

pub const METAPLEX_TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
