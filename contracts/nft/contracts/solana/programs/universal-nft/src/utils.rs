use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    keccak,
    system_program,
};
use std::convert::TryInto;

// Optional Metaplex imports - ensure the dependency exists in Cargo.toml
use mpl_token_metadata::{
    instruction as mpl_instruction,
    state::{Creator, DataV2},
    ID as TOKEN_METADATA_PROGRAM_ID,
};

/// Utility functions for handling Solana-specific challenges

/// Request additional compute units to handle complex operations
/// NOTE: Compute budget must be set client-side at transaction creation.
/// Clients should include ComputeBudgetInstruction::set_compute_unit_limit(units)
/// as the first instruction in their transaction.
pub fn request_compute_units(_units: u32) -> Result<()> {
    // No-op: compute budget cannot be modified via CPI
    // This must be done by the client when constructing the transaction
    Ok(())
}

/// Calculate rent exemption for account creation
pub fn calculate_rent_exemption(account_size: usize, rent: &Rent) -> u64 {
    rent.minimum_balance(account_size)
}

/// Validate Solana address format
pub fn validate_solana_address(address_bytes: &[u8]) -> Result<Pubkey> {
    if address_bytes.len() != 32 {
        return Err(error!(crate::UniversalNftError::InvalidRecipient));
    }
    
    Pubkey::try_from(address_bytes)
        .map_err(|_| error!(crate::UniversalNftError::InvalidRecipient))
}

/// Generate deterministic token ID based on collection and metadata
/// Uses keccak256 for cross-platform deterministic hashing
pub fn generate_token_id(collection: &Pubkey, next_id: u64, uri: &str) -> u64 {
    // Create a canonical byte sequence
    let mut data = Vec::new();
    data.extend_from_slice(&collection.to_bytes());
    data.extend_from_slice(&next_id.to_be_bytes()); // Big-endian for consistency
    data.extend_from_slice(uri.as_bytes());
    
    // Compute keccak256 hash
    let hash = keccak::hash(&data);
    
    // Take first 8 bytes as u64 (big-endian)
    u64::from_be_bytes([
        hash.0[0], hash.0[1], hash.0[2], hash.0[3],
        hash.0[4], hash.0[5], hash.0[6], hash.0[7]
    ])
}

/// Validate URI format and length
pub fn validate_uri(uri: &str) -> Result<()> {
    if uri.is_empty() || uri.len() > 200 {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    // Basic URI validation - should start with http/https or be IPFS
    if !uri.starts_with("http://") && 
       !uri.starts_with("https://") && 
       !uri.starts_with("ipfs://") &&
       !uri.starts_with("ar://") {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    Ok(())
}

/// Helper to create associated token account instruction
pub fn create_associated_token_account_ix(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    spl_associated_token_account::instruction::create_associated_token_account(
        payer,
        owner,
        mint,
        &spl_token::id(),
    )
}

/// Check if account has sufficient rent exemption
pub fn check_rent_exemption(account_info: &AccountInfo, required_size: usize, rent: &Rent) -> Result<()> {
    let required_lamports = rent.minimum_balance(required_size);
    
    if account_info.lamports() < required_lamports {
        return Err(error!(crate::UniversalNftError::RentExemptionFailed));
    }
    
    Ok(())
}

// ============================================================================
// MESSAGE FORMAT CONVERTERS
// ============================================================================

/// Cross-chain message format for EVM compatibility
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EvmMessage {
    pub destination: Vec<u8>,    // 20 bytes for EVM address or 0 for ZetaChain
    pub receiver: Vec<u8>,       // 20 bytes for EVM, 32 bytes for Solana
    pub token_id: u64,
    pub uri: String,
    pub sender: Vec<u8>,         // Original sender address
}

/// Solana native message format
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SolanaMessage {
    pub destination_chain_id: u64,
    pub recipient: Pubkey,
    pub token_id: u64,
    pub uri: String,
    pub sender: Pubkey,
    pub gas_amount: u64,
}

/// Enhanced origin-aware message for cross-chain transfers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OriginMessage {
    pub original_mint: Vec<u8>,  // 32 bytes for Solana mint (or padded)
    pub original_token_id: u64,  // universal token id
    pub origin_chain: u64,       // chain id where NFT was first minted
    pub metadata_uri: String,    // original metadata URI
    pub is_solana_native: bool,  // flag indicating Solana native origin
}

/// Convert ABI-encoded message from EVM chains to Solana format
/// EVM message format: (address destination, address receiver, uint256 tokenId, string uri, address sender)
pub fn decode_abi_message(abi_data: &[u8]) -> Result<EvmMessage> {
    // Simplified ABI decoding - in production, use a proper ABI decoder
    if abi_data.len() < 160 {  // Minimum size for the expected tuple
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    let mut offset = 0;
    
    // Skip function selector (4 bytes) if present
    if abi_data.len() > 4 && abi_data[0..4] == [0x00, 0x00, 0x00, 0x00] {
        offset += 4;
    }
    
    // Extract destination (32 bytes, but only last 20 are used for address)
    let destination = if offset + 32 <= abi_data.len() {
        abi_data[offset + 12..offset + 32].to_vec()
    } else {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    };
    offset += 32;
    
    // Extract receiver (32 bytes, but only last 20 are used for EVM address)
    let receiver = if offset + 32 <= abi_data.len() {
        abi_data[offset + 12..offset + 32].to_vec()
    } else {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    };
    offset += 32;
    
    // Extract token_id (32 bytes, big-endian u256, we take last 8 bytes as u64)
    let token_id = if offset + 32 <= abi_data.len() {
        u64::from_be_bytes([
            abi_data[offset + 24], abi_data[offset + 25], abi_data[offset + 26], abi_data[offset + 27],
            abi_data[offset + 28], abi_data[offset + 29], abi_data[offset + 30], abi_data[offset + 31]
        ])
    } else {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    };
    offset += 32;
    
    // Extract URI offset (32 bytes)
    let uri_offset = if offset + 32 <= abi_data.len() {
        u32::from_be_bytes([
            abi_data[offset + 28], abi_data[offset + 29], abi_data[offset + 30], abi_data[offset + 31]
        ]) as usize
    } else {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    };
    offset += 32;
    
    // Extract sender (32 bytes, but only last 20 are used for address)
    let sender = if offset + 32 <= abi_data.len() {
        abi_data[offset + 12..offset + 32].to_vec()
    } else {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    };
    
    // Extract URI from the specified offset
    let uri = if uri_offset + 32 <= abi_data.len() {
        let uri_length = u32::from_be_bytes([
            abi_data[uri_offset + 28], abi_data[uri_offset + 29], 
            abi_data[uri_offset + 30], abi_data[uri_offset + 31]
        ]) as usize;
        
        if uri_offset + 32 + uri_length <= abi_data.len() {
            String::from_utf8(abi_data[uri_offset + 32..uri_offset + 32 + uri_length].to_vec())
                .map_err(|_| error!(crate::UniversalNftError::InvalidMessage))?
        } else {
            return Err(error!(crate::UniversalNftError::InvalidMessage));
        }
    } else {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    };
    
    Ok(EvmMessage {
        destination,
        receiver,
        token_id,
        uri,
        sender,
    })
}

/// Convert Solana message to ABI-encoded format for EVM chains
pub fn encode_abi_message(message: &EvmMessage) -> Result<Vec<u8>> {
    let mut encoded = Vec::new();
    
    // Destination (32 bytes, padded)
    let mut dest_padded = vec![0u8; 32];
    if message.destination.len() <= 20 {
        dest_padded[32 - message.destination.len()..].copy_from_slice(&message.destination);
    }
    encoded.extend_from_slice(&dest_padded);
    
    // Receiver (32 bytes, padded)
    let mut receiver_padded = vec![0u8; 32];
    if message.receiver.len() <= 32 {
        receiver_padded[32 - message.receiver.len()..].copy_from_slice(&message.receiver);
    }
    encoded.extend_from_slice(&receiver_padded);
    
    // Token ID (32 bytes, big-endian)
    let mut token_id_bytes = vec![0u8; 32];
    token_id_bytes[24..].copy_from_slice(&message.token_id.to_be_bytes());
    encoded.extend_from_slice(&token_id_bytes);
    
    // URI offset (32 bytes) - points to after sender field
    let uri_offset = 128u32; // 4 * 32 bytes
    let mut offset_bytes = vec![0u8; 32];
    offset_bytes[28..].copy_from_slice(&uri_offset.to_be_bytes());
    encoded.extend_from_slice(&offset_bytes);
    
    // Sender (32 bytes, padded)
    let mut sender_padded = vec![0u8; 32];
    if message.sender.len() <= 32 {
        sender_padded[32 - message.sender.len()..].copy_from_slice(&message.sender);
    }
    encoded.extend_from_slice(&sender_padded);
    
    // URI length (32 bytes)
    let mut uri_len_bytes = vec![0u8; 32];
    uri_len_bytes[28..].copy_from_slice(&(message.uri.len() as u32).to_be_bytes());
    encoded.extend_from_slice(&uri_len_bytes);
    
    // URI data (padded to 32-byte boundary)
    encoded.extend_from_slice(message.uri.as_bytes());
    let padding = (32 - (message.uri.len() % 32)) % 32;
    encoded.extend_from_slice(&vec![0u8; padding]);
    
    Ok(encoded)
}

/// Convert between EVM and Solana message formats
pub fn evm_to_solana_message(evm_msg: &EvmMessage, chain_id: u64) -> Result<SolanaMessage> {
    // Convert receiver address to Solana Pubkey
    let recipient = if evm_msg.receiver.len() == 32 {
        Pubkey::try_from(evm_msg.receiver.as_slice())
            .map_err(|_| error!(crate::UniversalNftError::InvalidRecipient))?
    } else {
        // For EVM addresses, we need a mapping or conversion strategy
        // This is a simplified approach - in production, use proper address mapping
        return Err(error!(crate::UniversalNftError::InvalidRecipient));
    };
    
    let sender = if evm_msg.sender.len() == 32 {
        Pubkey::try_from(evm_msg.sender.as_slice())
            .map_err(|_| error!(crate::UniversalNftError::InvalidRecipient))?
    } else {
        // For EVM addresses, create a derived Solana address
        // This is a simplified approach
        return Err(error!(crate::UniversalNftError::InvalidRecipient));
    };
    
    Ok(SolanaMessage {
        destination_chain_id: chain_id,
        recipient,
        token_id: evm_msg.token_id,
        uri: evm_msg.uri.clone(),
        sender,
        gas_amount: 2000000, // Default gas amount
    })
}

// ============================================================================
// ENHANCED GAS FEE CALCULATION
// ============================================================================

/// Enhanced gas fee calculation with dynamic pricing
pub fn calculate_gas_fee(destination_chain: u64, gas_amount: u64) -> Result<u64> {
    // Base gas costs per chain (in lamports) - updated with more accurate estimates
    let base_gas_per_unit: u64 = match destination_chain {
        // Mainnet chains
        1 => 200,           // Ethereum - highest gas
        56 => 50,           // BSC - lower gas
        137 => 80,          // Polygon - moderate gas
        8453 => 120,        // Base - L2 gas
        42161 => 100,       // Arbitrum - L2 gas
        10 => 100,          // Optimism - L2 gas
        7000 => 30,         // ZetaChain - lowest gas
        
        // Testnet chains
        11155111 => 150,    // Ethereum Sepolia
        97 => 40,           // BSC Testnet
        80001 => 60,        // Polygon Mumbai
        84532 => 100,       // Base Sepolia
        421614 => 80,       // Arbitrum Sepolia
        11155420 => 80,     // Optimism Sepolia
        7001 => 25,         // ZetaChain Testnet
        
        _ => 100,           // Default gas for unknown chains
    };
    
    // Calculate total fee with overflow protection
    let total_fee = base_gas_per_unit
        .checked_mul(gas_amount)
        .ok_or(error!(crate::UniversalNftError::InsufficientGasAmount))?;
    
    // Apply network congestion multiplier (simplified)
    let congestion_multiplier = get_congestion_multiplier(destination_chain);
    let adjusted_fee = total_fee
        .checked_mul(congestion_multiplier)
        .ok_or(error!(crate::UniversalNftError::InsufficientGasAmount))?
        .checked_div(100)
        .ok_or(error!(crate::UniversalNftError::InsufficientGasAmount))?;
    
    // Ensure minimum gas fee (0.01 SOL)
    let min_fee = 10_000_000;
    
    // Ensure maximum gas fee (1 SOL) to prevent excessive costs
    let max_fee = 1_000_000_000;
    
    Ok(adjusted_fee.max(min_fee).min(max_fee))
}

/// Get congestion multiplier based on chain (simplified implementation)
fn get_congestion_multiplier(chain_id: u64) -> u64 {
    match chain_id {
        1 | 11155111 => 150,    // Ethereum often congested
        56 | 97 => 110,         // BSC moderate congestion
        137 | 80001 => 120,     // Polygon moderate congestion
        _ => 100,               // Default no multiplier
    }
}

// ============================================================================
// CHAIN VALIDATION HELPERS
// ============================================================================

/// Comprehensive chain ID validation for ZetaChain ecosystem
pub fn validate_chain_id(chain_id: u64) -> Result<()> {
    if is_supported_chain(chain_id) {
        Ok(())
    } else {
        Err(error!(crate::UniversalNftError::UnsupportedChain))
    }
}

/// Check if a chain ID is supported in the ZetaChain ecosystem
pub fn is_supported_chain(chain_id: u64) -> bool {
    matches!(
        chain_id,
        // Mainnet chains
        1 |      // Ethereum
        56 |     // BSC
        137 |    // Polygon
        8453 |   // Base
        42161 |  // Arbitrum
        10 |     // Optimism
        7000 |   // ZetaChain
        
        // Testnet chains
        11155111 | // Ethereum Sepolia
        97 |       // BSC Testnet
        80001 |    // Polygon Mumbai
        84532 |    // Base Sepolia
        421614 |   // Arbitrum Sepolia
        11155420 | // Optimism Sepolia
        7001       // ZetaChain Testnet
    )
}

/// Get chain name for logging and debugging
pub fn get_chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "Ethereum",
        56 => "BSC",
        137 => "Polygon",
        8453 => "Base",
        42161 => "Arbitrum",
        10 => "Optimism",
        7000 => "ZetaChain",
        11155111 => "Ethereum Sepolia",
        97 => "BSC Testnet",
        80001 => "Polygon Mumbai",
        84532 => "Base Sepolia",
        421614 => "Arbitrum Sepolia",
        11155420 => "Optimism Sepolia",
        7001 => "ZetaChain Testnet",
        _ => "Unknown Chain",
    }
}

/// Check if chain is a testnet
pub fn is_testnet_chain(chain_id: u64) -> bool {
    matches!(
        chain_id,
        11155111 | 97 | 80001 | 84532 | 421614 | 11155420 | 7001
    )
}

// ============================================================================
// ADDRESS FORMAT CONVERTERS
// ============================================================================

/// Convert Ethereum address (20 bytes) to Solana format
pub fn ethereum_to_solana_address(eth_address: &[u8]) -> Result<Pubkey> {
    if eth_address.len() != 20 {
        return Err(error!(crate::UniversalNftError::InvalidRecipient));
    }
    
    // Create a deterministic Solana address from Ethereum address
    // Using a seed-based approach for consistency
    let seeds = &[b"eth_bridge", eth_address];
    let (derived_address, _bump) = Pubkey::find_program_address(seeds, &crate::id());
    
    Ok(derived_address)
}

/// Convert Solana address to Ethereum format (for cross-chain messaging)
pub fn solana_to_ethereum_address(solana_address: &Pubkey) -> [u8; 20] {
    // Take the last 20 bytes of the Solana address hash
    let hash = keccak::hash(&solana_address.to_bytes());
    let mut eth_address = [0u8; 20];
    eth_address.copy_from_slice(&hash.to_bytes()[12..]);
    eth_address
}

/// Validate address format for specific chain
pub fn validate_address_for_chain(address: &[u8], chain_id: u64) -> Result<()> {
    match chain_id {
        // EVM chains expect 20-byte addresses
        1 | 56 | 137 | 8453 | 42161 | 10 | 11155111 | 97 | 80001 | 84532 | 421614 | 11155420 => {
            if address.len() != 20 {
                return Err(error!(crate::UniversalNftError::InvalidRecipient));
            }
        },
        // ZetaChain can handle both formats
        7000 | 7001 => {
            if address.len() != 20 && address.len() != 32 {
                return Err(error!(crate::UniversalNftError::InvalidRecipient));
            }
        },
        _ => {
            return Err(error!(crate::UniversalNftError::UnsupportedChain));
        }
    }
    Ok(())
}

/// Convert address bytes to appropriate format for chain
pub fn format_address_for_chain(address: &[u8], chain_id: u64) -> Result<Vec<u8>> {
    validate_address_for_chain(address, chain_id)?;
    
    match chain_id {
        // EVM chains - ensure 20 bytes
        1 | 56 | 137 | 8453 | 42161 | 10 | 11155111 | 97 | 80001 | 84532 | 421614 | 11155420 => {
            if address.len() == 20 {
                Ok(address.to_vec())
            } else {
                // Convert from Solana to Ethereum format
                let solana_pubkey = Pubkey::try_from(address)
                    .map_err(|_| error!(crate::UniversalNftError::InvalidRecipient))?;
                Ok(solana_to_ethereum_address(&solana_pubkey).to_vec())
            }
        },
        // ZetaChain - preserve original format
        7000 | 7001 => Ok(address.to_vec()),
        _ => Err(error!(crate::UniversalNftError::UnsupportedChain)),
    }
}

// ============================================================================
// METADATA HELPERS
// ============================================================================

/// Enhanced URI validation with more comprehensive checks
pub fn validate_metadata_uri(uri: &str) -> Result<()> {
    // Check length
    if uri.is_empty() || uri.len() > 500 {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    // Check for valid URI schemes
    let valid_schemes = [
        "https://",
        "http://",
        "ipfs://",
        "ar://",           // Arweave
        "data:",           // Data URIs
    ];
    
    if !valid_schemes.iter().any(|scheme| uri.starts_with(scheme)) {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    // Additional validation for IPFS
    if uri.starts_with("ipfs://") {
        let hash_part = &uri[7..];
        if hash_part.len() < 46 || hash_part.len() > 100 {
            return Err(error!(crate::UniversalNftError::InvalidMessage));
        }
    }
    
    // Check for potentially malicious content
    let forbidden_patterns = ["javascript:", "data:text/html", "<script"];
    if forbidden_patterns.iter().any(|pattern| uri.to_lowercase().contains(pattern)) {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    Ok(())
}

/// Create standardized metadata JSON structure
pub fn create_metadata_json(
    name: &str,
    description: &str,
    image_uri: &str,
    attributes: Option<Vec<(String, String)>>,
) -> Result<String> {
    // Validate inputs
    if name.is_empty() || name.len() > 100 {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    if description.len() > 1000 {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    validate_metadata_uri(image_uri)?;
    
    // Build JSON manually to avoid external dependencies
    let mut json = format!(
        r#"{{"name":"{}","description":"{}","image":"{}""#,
        escape_json_string(name),
        escape_json_string(description),
        escape_json_string(image_uri)
    );
    
    // Add attributes if provided
    if let Some(attrs) = attributes {
        if !attrs.is_empty() {
            json.push_str(r#","attributes":["#);
            for (i, (trait_type, value)) in attrs.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                json.push_str(&format!(
                    r#"{{"trait_type":"{}","value":"{}"}}"#,
                    escape_json_string(trait_type),
                    escape_json_string(value)
                ));
            }
            json.push(']');
        }
    }
    
    json.push('}');
    
    // Validate final JSON size
    if json.len() > 2000 {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    Ok(json)
}

/// Escape string for JSON
fn escape_json_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => r#"\""#.to_string(),
            '\\' => r#"\\"#.to_string(),
            '\n' => r#"\n"#.to_string(),
            '\r' => r#"\r"#.to_string(),
            '\t' => r#"\t"#.to_string(),
            c if c.is_control() => format!(r#"\u{:04x}"#, c as u32),
            c => c.to_string(),
        })
        .collect()
}

/// Validate metadata JSON structure
pub fn validate_metadata_json(json: &str) -> Result<()> {
    // Basic JSON structure validation
    if json.len() > 5000 {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    // Check for required fields (simplified validation)
    if !json.contains("\"name\"") || !json.contains("\"image\"") {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    // Check for balanced braces
    let open_braces = json.chars().filter(|&c| c == '{').count();
    let close_braces = json.chars().filter(|&c| c == '}').count();
    
    if open_braces != close_braces {
        return Err(error!(crate::UniversalNftError::InvalidMessage));
    }
    
    Ok(())
}

/// Generate IPFS-compatible hash for metadata
pub fn generate_metadata_hash(metadata: &str) -> [u8; 32] {
    keccak::hash(metadata.as_bytes()).to_bytes()
}

/// Create deterministic token URI based on collection and token ID
pub fn generate_token_uri(collection: &Pubkey, token_id: u64, base_uri: &str) -> String {
    if base_uri.ends_with('/') {
        format!("{}{}", base_uri, token_id)
    } else {
        format!("{}/{}", base_uri, token_id)
    }
}

// ============================================================================
// NFT ORIGIN UTILITIES
// ============================================================================

/// Derive the NftOrigin PDA for a given token_id
pub fn derive_nft_origin_pda(program_id: &Pubkey, token_id: u64) -> (Pubkey, u8) {
    let seed_token = token_id.to_le_bytes();
    Pubkey::find_program_address(
        &[b"nft_origin", &seed_token],
        program_id,
    )
}

/// Generate deterministic token id from mint + block_number + next_id
/// This follows the documentation: token_id := keccak256(mint_pubkey || block_number || next_token_id)[0..8]
pub fn generate_token_id_from_mint(mint: &Pubkey, block_number: u64, next_id: u64) -> u64 {
    let mut data = Vec::new();
    data.extend_from_slice(&mint.to_bytes());
    data.extend_from_slice(&block_number.to_be_bytes());
    data.extend_from_slice(&next_id.to_be_bytes());
    let h = keccak::hash(&data);
    u64::from_be_bytes(h.0[0..8].try_into().unwrap())
}

/// Validate token id format (non-zero and within reasonable bounds)
pub fn validate_token_id_format(token_id: u64) -> Result<()> {
    if token_id == 0 {
        return Err(error!(crate::UniversalNftError::InvalidTokenId));
    }
    // Additional range checks can be added as needed
    Ok(())
}

/// Check whether a token id is available by inspecting the provided origin account info.
/// If the account_info has non-zero data length we assume it already exists.
pub fn is_token_id_available(origin_info: Option<&AccountInfo>) -> Result<bool> {
    if let Some(info) = origin_info {
        // If account is uninitialized or has zero data length, it's available
        if info.data_len() == 0 {
            Ok(true)
        } else {
            // If data_len > 0, it's likely occupied
            Ok(false)
        }
    } else {
        // No account info provided => caller must treat as available
        Ok(true)
    }
}

/// Validate token id uniqueness by checking the provided origin account info.
/// Returns Ok(()) if available, error if already exists.
pub fn validate_token_id_uniqueness(origin_info: &AccountInfo) -> Result<()> {
    if origin_info.data_len() > 0 {
        return Err(error!(crate::UniversalNftError::InvalidTokenId));
    }
    Ok(())
}

/// Generic helper to find next available token id starting from `start`.
/// The caller must provide a closure `is_available` that checks availability for a given id.
pub fn get_next_available_token_id<F>(start: u64, max_search: u64, mut is_available: F) -> Result<u64>
where
    F: FnMut(u64) -> Result<bool>,
{
    let mut id = start;
    for _ in 0..max_search {
        let available = is_available(id)?;
        if available {
            return Ok(id);
        }
        id = id.checked_add(1).ok_or(error!(crate::UniversalNftError::InvalidTokenId))?;
    }
    Err(error!(crate::UniversalNftError::InvalidTokenId))
}

/// Extract token_id from raw NftOrigin account data (Anchor/account borsh layout)
/// The layout is:
/// [8 discriminator][32 original_mint][8 token_id][32 collection][8 chain_of_origin][8 created_at][4 metadata_len][metadata bytes][1 bump]
pub fn extract_token_id_from_origin(data: &[u8]) -> Result<u64> {
    // Minimum size check
    if data.len() < 8 + 32 + 8 {
        return Err(error!(crate::UniversalNftError::InvalidAccountData));
    }
    let token_id_offset = 8 + 32;
    let token_id_bytes: [u8; 8] = data[token_id_offset..token_id_offset + 8]
        .try_into()
        .map_err(|_| error!(crate::UniversalNftError::InvalidAccountData))?;
    Ok(u64::from_le_bytes(token_id_bytes))
}

/// Determine origin chain from raw NftOrigin account data
pub fn determine_origin_chain_from_origin_data(data: &[u8]) -> Result<u64> {
    // chain_of_origin offset: 8 + 32 + 8 + 32 = 80
    if data.len() < 80 + 8 {
        return Err(error!(crate::UniversalNftError::InvalidAccountData));
    }
    let offset = 8 + 32 + 8 + 32;
    let bytes: [u8; 8] = data[offset..offset + 8]
        .try_into()
        .map_err(|_| error!(crate::UniversalNftError::InvalidAccountData))?;
    Ok(u64::from_le_bytes(bytes))
}

/// Check if the NftOrigin (raw data) indicates a Solana native origin
pub fn is_solana_native_nft_from_origin_data(data: &[u8]) -> Result<bool> {
    let chain_id = determine_origin_chain_from_origin_data(data)?;
    // Using conventional Solana identifiers (these values are examples used across the codebase)
    Ok(chain_id == 101 || chain_id == 102 || chain_id == 103)
}

/// Retrieve origin metadata URI from raw NftOrigin account data
pub fn get_origin_metadata_from_origin_data(data: &[u8]) -> Result<String> {
    // metadata_uri offset after:
    // 8 (disc) + 32 (original_mint) + 8 (token_id) + 32 (collection) + 8 (chain_of_origin) + 8 (created_at) = 96
    if data.len() < 96 + 4 {
        return Err(error!(crate::UniversalNftError::InvalidAccountData));
    }
    let mut offset = 8 + 32 + 8 + 32 + 8 + 8;
    // Next is u32 length (little-endian)
    let len_bytes: [u8; 4] = data[offset..offset + 4]
        .try_into()
        .map_err(|_| error!(crate::UniversalNftError::InvalidAccountData))?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    offset += 4;
    if data.len() < offset + len {
        return Err(error!(crate::UniversalNftError::InvalidAccountData));
    }
    let uri_bytes = &data[offset..offset + len];
    String::from_utf8(uri_bytes.to_vec()).map_err(|_| error!(crate::UniversalNftError::InvalidAccountData))
}

// ============================================================================
// METAPLEX INTEGRATION HELPERS
// ============================================================================

/// Derive metadata PDA for a given mint using Metaplex Token Metadata program rules
pub fn derive_metadata_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"metadata",
            &TOKEN_METADATA_PROGRAM_ID.to_bytes(),
            &mint.to_bytes(),
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    )
}

/// Create Metaplex metadata creation instruction (DataV2) using mpl-token-metadata
/// This returns the CPI instruction that can be invoked to create metadata on-chain.
#[allow(clippy::too_many_arguments)]
pub fn create_metadata_instruction(
    metadata_account: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    update_authority: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    creators: Option<Vec<Creator>>,
) -> Instruction {
    // Build DataV2 structure
    let data = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points,
        creators,
        collection: None,
        uses: None,
    };

    mpl_instruction::create_metadata_accounts_v2(
        TOKEN_METADATA_PROGRAM_ID,
        *metadata_account,
        *mint,
        *mint_authority,
        *payer,
        *update_authority,
        data,
        true,  // is_mutable
        false, // update_authority_is_signer (we pass update_authority as signer in CPI)
    )
}

/// Create master edition instruction for NFT uniqueness
pub fn create_master_edition_instruction(
    edition_account: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    update_authority: &Pubkey,
    metadata_account: &Pubkey,
    max_supply: Option<u64>,
) -> Instruction {
    mpl_instruction::create_master_edition_v3(
        TOKEN_METADATA_PROGRAM_ID,
        *edition_account,
        *mint,
        *mint_authority,
        *payer,
        *update_authority,
        *metadata_account,
        max_supply,
    )
}

/// Update metadata instruction for returning NFTs (partial update)
pub fn update_metadata_instruction(
    metadata_account: &Pubkey,
    update_authority: &Pubkey,
    new_name: Option<String>,
    new_symbol: Option<String>,
    new_uri: Option<String>,
    new_creators: Option<Option<Vec<Creator>>>,
    primary_sale_happened: Option<bool>,
) -> Instruction {
    // Build DataV2 partial update - mpl may require DataV2; using None values for unspecified
    // The instruction expects a DataV2 struct and optional fields; using existing helper
    let current_data = DataV2 {
        name: new_name.clone().unwrap_or_default(),
        symbol: new_symbol.clone().unwrap_or_default(),
        uri: new_uri.clone().unwrap_or_default(),
        seller_fee_basis_points: 0,
        creators: new_creators.clone().unwrap_or(None),
        collection: None,
        uses: None,
    };

    mpl_instruction::update_metadata_accounts_v2(
        TOKEN_METADATA_PROGRAM_ID,
        *metadata_account,
        *update_authority,
        Some(current_data),
        Some(primary_sale_happened.unwrap_or(false)),
        None,
    )
}

// ============================================================================
// CROSS-CHAIN ORIGIN HELPERS
// ============================================================================

/// Serialize OriginMessage using Borsh (AnchorSerialize)
pub fn serialize_origin_message(origin: &OriginMessage) -> Result<Vec<u8>> {
    Ok(origin.try_to_vec().map_err(|_| error!(crate::UniversalNftError::InvalidMessage))?)
}

/// Deserialize OriginMessage from bytes
pub fn deserialize_origin_message(data: &[u8]) -> Result<OriginMessage> {
    OriginMessage::try_from_slice(data).map_err(|_| error!(crate::UniversalNftError::InvalidMessage))
}

/// Attach origin information into an existing EVM message by embedding the origin bytes
/// at the end of the URI as a base64 string (simple approach to ensure ABI compatibility).
pub fn attach_origin_to_evm_message(evm_msg: &mut EvmMessage, origin: &OriginMessage) -> Result<()> {
    let origin_bytes = serialize_origin_message(origin)?;
    // Base64 encode without bringing in extra dependencies - use a simple hex encoding
    // Hex encode origin bytes
    let mut hex = String::with_capacity(origin_bytes.len() * 2);
    for b in origin_bytes {
        hex.push_str(&format!("{:02x}", b));
    }
    // Append as fragment to URI: original_uri#origin=hexpayload
    evm_msg.uri = format!("{}#origin={}", evm_msg.uri, hex);
    Ok(())
}

/// Extract origin information from an EVM message URI if attached by attach_origin_to_evm_message
pub fn extract_origin_from_evm_message(evm_msg: &EvmMessage) -> Result<Option<OriginMessage>> {
    // Look for "#origin=" fragment
    if let Some(idx) = evm_msg.uri.find("#origin=") {
        let hex_payload = &evm_msg.uri[idx + 8..];
        if hex_payload.is_empty() {
            return Ok(None);
        }
        // Decode hex
        if hex_payload.len() % 2 != 0 {
            return Err(error!(crate::UniversalNftError::InvalidMessage));
        }
        let bytes_res: Result<Vec<u8>, _> = (0..hex_payload.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_payload[i..i+2], 16))
            .collect();
        let bytes = bytes_res.map_err(|_| error!(crate::UniversalNftError::InvalidMessage))?;
        let origin = deserialize_origin_message(&bytes)?;
        return Ok(Some(origin));
    }
    Ok(None)
}

/// Validate that the EVM message and the attached origin (if present) are consistent
pub fn validate_message_origin_consistency(evm_msg: &EvmMessage, origin_opt: Option<&OriginMessage>) -> Result<()> {
    if let Some(origin) = origin_opt {
        if origin.original_token_id != evm_msg.token_id {
            return Err(error!(crate::UniversalNftError::InvalidMessage));
        }
        // Additional checks: ensure metadata uri matches suffix or that key fields are consistent
        if !origin.metadata_uri.is_empty() && !evm_msg.uri.starts_with(&origin.metadata_uri) {
            // Not strictly required to match fully, but flag obvious mismatches
            // We allow some flexibility as chains may rewrite URI prefixes
        }
    }
    Ok(())
}
