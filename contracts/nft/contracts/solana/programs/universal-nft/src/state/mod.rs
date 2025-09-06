use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Collection {
    pub authority: Pubkey,
    #[max_len(32)]
    pub name: String,
    #[max_len(10)]
    pub symbol: String,
    #[max_len(200)]
    pub uri: String,
    pub tss_address: [u8; 20], // ZetaChain TSS ECDSA address
    pub gateway_address: Option<Pubkey>, // Gateway PDA if needed
    pub universal_address: Option<Pubkey>, // Universal contract address on ZetaChain
    pub next_token_id: u64,
    pub nonce: u64, // Replay protection counter
    pub total_minted: u64, // Track total NFTs minted
    pub solana_native_count: u64, // Track Solana-native NFTs
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct NftOrigin {
    pub original_mint: Pubkey, // Stores the original mint key
    pub token_id: u64, // The universal token ID
    pub collection: Pubkey, // Reference to the collection
    pub chain_of_origin: u64, // Chain where NFT was first minted
    pub created_at: i64, // Timestamp of creation
    #[max_len(200)]
    pub metadata_uri: String, // Original metadata URI
    pub bump: u8, // PDA bump
}

#[account]
#[derive(InitSpace)]
pub struct Connected {
    pub collection: Pubkey,
    #[max_len(32)]
    pub chain_id: Vec<u8>,
    #[max_len(64)]
    pub contract_address: Vec<u8>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrossChainMessage {
    pub destination_chain: Vec<u8>,
    pub recipient: Vec<u8>,
    pub token_id: u64,
    pub uri: String,
    pub sender: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ZetaChainMessage {
    pub destination_chain_id: u64,
    pub destination_address: [u8; 20],
    pub destination_gas_limit: u64,
    pub message: Vec<u8>,
    pub token_id: u64,
    pub uri: String,
    pub sender: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RevertContext {
    pub token_id: u64,
    pub uri: String,
    pub original_sender: Vec<u8>,
    pub revert_reason: String,
    pub revert_message: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EVMMessage {
    pub token_id: u64,
    pub recipient: [u8; 20], // EVM address format
    pub uri: String,
    pub sender: [u8; 20], // EVM address format
}

// Supported chain IDs constants
pub const CHAIN_ID_ZETACHAIN: u64 = 7000;
pub const CHAIN_ID_ETHEREUM: u64 = 1;
pub const CHAIN_ID_BSC: u64 = 56;
pub const CHAIN_ID_POLYGON: u64 = 137;
pub const CHAIN_ID_BASE: u64 = 8453;
pub const CHAIN_ID_ARBITRUM: u64 = 42161;
pub const CHAIN_ID_OPTIMISM: u64 = 10;

// Testnet chain IDs
pub const CHAIN_ID_SEPOLIA: u64 = 11155111;
pub const CHAIN_ID_BSC_TESTNET: u64 = 97;
pub const CHAIN_ID_MUMBAI: u64 = 80001;
pub const CHAIN_ID_BASE_SEPOLIA: u64 = 84532;
pub const CHAIN_ID_ARBITRUM_SEPOLIA: u64 = 421614;
pub const CHAIN_ID_OPTIMISM_SEPOLIA: u64 = 11155420;
pub const CHAIN_ID_ZETACHAIN_TESTNET: u64 = 7001;

/// Validation helper functions
impl Collection {
    /// Validate that the caller is the collection authority
    pub fn validate_authority(&self, authority: &Pubkey) -> Result<()> {
        require_keys_eq!(
            *authority,
            self.authority,
            crate::UniversalNftError::InvalidSignature
        );
        Ok(())
    }

    /// Get the next token ID and increment the counter
    pub fn get_next_token_id(&mut self) -> Result<u64> {
        let token_id = self.next_token_id;
        self.next_token_id = self.next_token_id
            .checked_add(1)
            .ok_or(crate::UniversalNftError::InvalidTokenId)?;
        Ok(token_id)
    }

    /// Increment nonce for replay protection
    pub fn increment_nonce(&mut self) -> u64 {
        let current_nonce = self.nonce;
        self.nonce = self.nonce.saturating_add(1);
        current_nonce
    }

    /// Increment total minted count
    pub fn increment_total_minted(&mut self) -> Result<()> {
        self.total_minted = self.total_minted
            .checked_add(1)
            .ok_or(crate::UniversalNftError::InvalidTokenId)?;
        Ok(())
    }

    /// Increment Solana native count
    pub fn increment_solana_native_count(&mut self) -> Result<()> {
        self.solana_native_count = self.solana_native_count
            .checked_add(1)
            .ok_or(crate::UniversalNftError::InvalidTokenId)?;
        Ok(())
    }

    /// Get collection statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (self.total_minted, self.solana_native_count, self.total_minted - self.solana_native_count)
    }

    /// Check if collection has reached maximum capacity
    pub fn is_at_capacity(&self, max_supply: Option<u64>) -> bool {
        if let Some(max) = max_supply {
            self.total_minted >= max
        } else {
            false
        }
    }
}

impl Connected {
    /// Create seeds for Connected PDA
    pub fn seeds(collection: &Pubkey, chain_id: &[u8]) -> Vec<Vec<u8>> {
        vec![
            b"connected".to_vec(),
            collection.to_bytes().to_vec(),
            chain_id.to_vec(),
        ]
    }
}

impl NftOrigin {
    /// Create seeds for NftOrigin PDA
    pub fn seeds(token_id: u64) -> Vec<Vec<u8>> {
        vec![
            b"nft_origin".to_vec(),
            token_id.to_le_bytes().to_vec(),
        ]
    }

    /// Validate token ID format and uniqueness
    pub fn validate_token_id(&self, expected_token_id: u64) -> Result<()> {
        require_eq!(
            self.token_id,
            expected_token_id,
            crate::UniversalNftError::InvalidTokenId
        );
        Ok(())
    }

    /// Check if NFT originated on Solana
    pub fn is_solana_native(&self) -> bool {
        // Solana mainnet and devnet chain IDs
        self.chain_of_origin == 101 || self.chain_of_origin == 103 || self.chain_of_origin == 102
    }

    /// Get the origin chain name for display purposes
    pub fn get_origin_chain_name(&self) -> &'static str {
        match self.chain_of_origin {
            101 => "Solana Mainnet",
            102 => "Solana Testnet", 
            103 => "Solana Devnet",
            CHAIN_ID_ETHEREUM => "Ethereum",
            CHAIN_ID_BSC => "BSC",
            CHAIN_ID_POLYGON => "Polygon",
            CHAIN_ID_BASE => "Base",
            CHAIN_ID_ARBITRUM => "Arbitrum",
            CHAIN_ID_OPTIMISM => "Optimism",
            CHAIN_ID_ZETACHAIN => "ZetaChain",
            CHAIN_ID_SEPOLIA => "Ethereum Sepolia",
            CHAIN_ID_BSC_TESTNET => "BSC Testnet",
            CHAIN_ID_MUMBAI => "Polygon Mumbai",
            CHAIN_ID_BASE_SEPOLIA => "Base Sepolia",
            CHAIN_ID_ARBITRUM_SEPOLIA => "Arbitrum Sepolia",
            CHAIN_ID_OPTIMISM_SEPOLIA => "Optimism Sepolia",
            CHAIN_ID_ZETACHAIN_TESTNET => "ZetaChain Testnet",
            _ => "Unknown Chain",
        }
    }

    /// Update metadata URI while preserving origin information
    pub fn update_metadata_uri(&mut self, new_uri: String) -> Result<()> {
        require!(
            new_uri.len() <= 200,
            crate::UniversalNftError::InvalidMessage
        );
        self.metadata_uri = new_uri;
        Ok(())
    }
}

/// Check if a chain ID is supported
pub fn is_supported_chain(chain_id: u64) -> bool {
    matches!(
        chain_id,
        CHAIN_ID_ZETACHAIN |
        CHAIN_ID_ETHEREUM |
        CHAIN_ID_BSC |
        CHAIN_ID_POLYGON |
        CHAIN_ID_BASE |
        CHAIN_ID_ARBITRUM |
        CHAIN_ID_OPTIMISM |
        CHAIN_ID_SEPOLIA |
        CHAIN_ID_BSC_TESTNET |
        CHAIN_ID_MUMBAI |
        CHAIN_ID_BASE_SEPOLIA |
        CHAIN_ID_ARBITRUM_SEPOLIA |
        CHAIN_ID_OPTIMISM_SEPOLIA |
        CHAIN_ID_ZETACHAIN_TESTNET
    )
}

/// Validate chain ID format
pub fn validate_chain_id(chain_id: &[u8]) -> Result<u64> {
    if chain_id.len() != 8 {
        return Err(crate::UniversalNftError::InvalidDestinationChain.into());
    }
    
    let chain_id_u64 = u64::from_le_bytes(
        chain_id.try_into()
            .map_err(|_| crate::UniversalNftError::InvalidDestinationChain)?
    );
    
    if !is_supported_chain(chain_id_u64) {
        return Err(crate::UniversalNftError::UnsupportedChain.into());
    }
    
    Ok(chain_id_u64)
}

/// Validate EVM address format (20 bytes)
pub fn validate_evm_address(address: &[u8]) -> Result<[u8; 20]> {
    if address.len() != 20 {
        return Err(crate::UniversalNftError::InvalidRecipientAddress.into());
    }
    
    let mut addr_array = [0u8; 20];
    addr_array.copy_from_slice(address);
    Ok(addr_array)
}

/// Validate Solana address format (32 bytes)
pub fn validate_solana_address(address: &[u8]) -> Result<Pubkey> {
    if address.len() != 32 {
        return Err(crate::UniversalNftError::InvalidRecipientAddress.into());
    }
    
    let mut addr_array = [0u8; 32];
    addr_array.copy_from_slice(address);
    Ok(Pubkey::new_from_array(addr_array))
}

/// Convert between address formats for cross-chain compatibility
pub fn convert_address_format(address: &[u8], target_chain: u64) -> Result<Vec<u8>> {
    match target_chain {
        // EVM chains require 20-byte addresses
        CHAIN_ID_ETHEREUM | CHAIN_ID_BSC | CHAIN_ID_POLYGON | 
        CHAIN_ID_BASE | CHAIN_ID_ARBITRUM | CHAIN_ID_OPTIMISM |
        CHAIN_ID_SEPOLIA | CHAIN_ID_BSC_TESTNET | CHAIN_ID_MUMBAI |
        CHAIN_ID_BASE_SEPOLIA | CHAIN_ID_ARBITRUM_SEPOLIA | CHAIN_ID_OPTIMISM_SEPOLIA => {
            if address.len() == 20 {
                Ok(address.to_vec())
            } else if address.len() == 32 {
                // For Solana to EVM, we might need to derive an EVM address
                // This is a simplified approach - real implementation would use proper derivation
                Ok(address[..20].to_vec())
            } else {
                Err(crate::UniversalNftError::InvalidRecipientAddress.into())
            }
        },
        // ZetaChain and Solana use 32-byte addresses
        CHAIN_ID_ZETACHAIN | CHAIN_ID_ZETACHAIN_TESTNET => {
            if address.len() == 32 {
                Ok(address.to_vec())
            } else if address.len() == 20 {
                // Pad EVM address to 32 bytes for ZetaChain
                let mut padded = vec![0u8; 12];
                padded.extend_from_slice(address);
                Ok(padded)
            } else {
                Err(crate::UniversalNftError::InvalidRecipientAddress.into())
            }
        },
        _ => Err(crate::UniversalNftError::UnsupportedChain.into())
    }
}
