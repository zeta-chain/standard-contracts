// ========== PROGRAM CONFIGURATION ==========
/// Maximum length for NFT metadata URI
pub const MAX_URI_LENGTH: usize = 200;

/// Maximum length for NFT name
pub const MAX_NAME_LENGTH: usize = 32;

/// Maximum length for NFT symbol
pub const MAX_SYMBOL_LENGTH: usize = 10;

/// Maximum length for cross-chain final recipient field to cap message size
pub const MAX_RECIPIENT_LENGTH: usize = 128;

/// Maximum length for collection name (keeping for potential future use)
pub const MAX_COLLECTION_NAME_LENGTH: usize = 50;

// ========== PDA SEEDS ==========
/// Seed for NFT origin PDA
pub const NFT_ORIGIN_SEED: &[u8] = b"nft_origin";

/// Seed for universal NFT config PDA
pub const UNIVERSAL_NFT_CONFIG_SEED: &[u8] = b"connected";

/// Seed for mint reservation ticket PDA
pub const MINT_TICKET_SEED: &[u8] = b"mint_ticket";

/// Seed for NFT collection PDA (removed - no longer using collections)
// pub const NFT_COLLECTION_SEED: &[u8] = b"nft_collection";

/// Seed for metadata account
pub const METADATA_SEED: &[u8] = b"metadata";

/// Seed for master edition account
pub const MASTER_EDITION_SEED: &[u8] = b"edition";

// ========== CROSS-CHAIN CONFIGURATION ==========
/// Maximum number of supported chains
pub const MAX_SUPPORTED_CHAINS: usize = 10;

/// Chain ID for ZetaChain
pub const ZETACHAIN_CHAIN_ID: u64 = 7000;

/// Chain ID for Ethereum
pub const ETHEREUM_CHAIN_ID: u64 = 1;

/// Chain ID for BNB Chain
pub const BNB_CHAIN_ID: u64 = 56;

/// Chain ID for Base
pub const BASE_CHAIN_ID: u64 = 8453;

/// Chain ID for Solana (for cross-chain messaging)
pub const SOLANA_CHAIN_ID: u64 = 10001;

// ========== TIMING CONSTANTS ==========
/// Grace period for cross-chain operations (seconds)
pub const CROSS_CHAIN_GRACE_PERIOD: i64 = 300; // 5 minutes

/// Maximum age for cross-chain messages (seconds)
pub const MAX_MESSAGE_AGE: i64 = 3600; // 1 hour

// ========== SECURITY CONSTANTS ==========
/// Maximum number of retries for failed operations
pub const MAX_RETRY_ATTEMPTS: u8 = 3;

/// Minimum time between operations (anti-spam)
pub const MIN_OPERATION_INTERVAL: i64 = 1; // 1 second

// ========== METAPLEX INTEGRATION ==========
use anchor_lang::solana_program::{pubkey, pubkey::Pubkey};

/// Metaplex Token Metadata Program ID
pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

/// Seller fee basis points (0 = no royalties)
pub const DEFAULT_SELLER_FEE_BASIS_POINTS: u16 = 0;

/// Primary sale happened flag
pub const DEFAULT_PRIMARY_SALE_HAPPENED: bool = false;

/// Is mutable flag for metadata
pub const DEFAULT_IS_MUTABLE: bool = true;

/// Estimated sizes (bytes) for Metaplex accounts used to preflight rent checks.
/// These are conservative upper bounds to avoid partial state when funds are insufficient.
pub const EST_METADATA_ACCOUNT_SIZE: usize = 1024; // conservative upper bound
pub const EST_MASTER_EDITION_ACCOUNT_SIZE: usize = 512; // conservative upper bound

// ========== GAS AND FEE CONSTANTS ==========
// Removed gas limit constants since Solana -> ZetaChain doesn't require gas specification

// ========== VALIDATION CONSTANTS ==========
/// Minimum token ID length (in bytes)
pub const MIN_TOKEN_ID_LENGTH: usize = 8;

/// Maximum token ID length (in bytes)
pub const MAX_TOKEN_ID_LENGTH: usize = 32;

/// Standard token ID length (256 bits = 32 bytes)
pub const STANDARD_TOKEN_ID_LENGTH: usize = 32;

// ========== ZETACHAIN GATEWAY CONSTANTS ==========
/// ZetaChain Gateway Program ID (placeholder - replace with actual deployed gateway)
/// Note: This should be updated with the real ZetaChain gateway program ID for production
/// For devnet, use: "94U5AHQMKkV5txNJ17QPXWoh474PheGou6cNP2FEuL1d"
/// For mainnet, use: "ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis"
pub const ZETACHAIN_GATEWAY_PROGRAM_ID: Pubkey = pubkey!("94U5AHQMKkV5txNJ17QPXWoh474PheGou6cNP2FEuL1d");
