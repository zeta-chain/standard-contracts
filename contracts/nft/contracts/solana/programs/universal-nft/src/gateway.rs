use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    system_instruction,
    keccak,
};
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::{
    UniversalNftError,
    ZETACHAIN_GATEWAY_PROGRAM_ID,
    GATEWAY_PDA_SEED,
    get_current_chain_id,
};

/// ZetaChain Gateway integration module
/// Handles all cross-chain interactions through the ZetaChain Gateway program

/// Gateway instruction types for ZetaChain
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ZetaGatewayInstruction {
    /// Deposit SOL to ZetaChain
    Deposit {
        amount: u64,
        receiver: [u8; 20],
        revert_options: Option<RevertOptions>,
    },
    /// Deposit and call a universal app on ZetaChain
    DepositAndCall {
        amount: u64,
        receiver: [u8; 20],
        message: Vec<u8>,
        revert_options: Option<RevertOptions>,
    },
    /// Call a universal app without depositing
    Call {
        receiver: [u8; 20],
        message: Vec<u8>,
        call_options: CallOptions,
    },
}

/// Revert options for failed cross-chain calls
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RevertOptions {
    pub revert_address: [u8; 20],
    pub call_on_revert: bool,
    pub abort_address: [u8; 20],
    pub revert_message: Vec<u8>,
    pub on_revert_gas_limit: u64,
}

/// Call options for gateway calls
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CallOptions {
    pub gas_limit: u64,
    pub is_arbitrary_call: bool,
}

/// Gateway PDA state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GatewayPDA {
    pub nonce: u64,
    pub tss_address: [u8; 20],
    pub authority: Pubkey,
    pub chain_id: u64,
    pub bump: u8,
}

/// Cross-chain message formats for different destination chains
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum GatewayCrossChainMessage {
    /// EVM-compatible message format
    EVM(EVMCrossChainMessage),
    /// ZetaChain native message format
    ZetaChain(ZetaChainCrossChainMessage),
    /// Bitcoin message format
    Bitcoin(BitcoinCrossChainMessage),
    /// Generic message format for other chains
    Generic(GenericCrossChainMessage),
}

/// EVM cross-chain message structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EVMCrossChainMessage {
    pub token_id: u64,
    pub recipient: [u8; 20],
    pub uri: String,
    pub sender: [u8; 20],
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
    pub metadata_hash: [u8; 32],
    pub collection_address: [u8; 20],
    pub chain_id: u64,
}

/// ZetaChain cross-chain message structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ZetaChainCrossChainMessage {
    pub destination_chain_id: u64,
    pub destination_address: [u8; 20],
    pub destination_gas_limit: u64,
    pub token_id: u64,
    pub uri: String,
    pub sender: [u8; 32],
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
    pub call_data: Vec<u8>,
}

/// Bitcoin cross-chain message structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BitcoinCrossChainMessage {
    pub token_id: u64,
    pub recipient: Vec<u8>, // Bitcoin address can vary in length
    pub uri: String,
    pub sender: [u8; 32],
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
    pub memo: String,
}

/// Generic cross-chain message structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GenericCrossChainMessage {
    pub destination_chain: u64,
    pub recipient: Vec<u8>,
    pub token_id: u64,
    pub uri: String,
    pub sender: Vec<u8>,
    pub origin_chain: u64,
    pub original_mint: [u8; 32],
    pub is_solana_native: bool,
    pub custom_data: Vec<u8>,
}

/// Gateway integration functions
impl GatewayCrossChainMessage {
    /// Create a cross-chain message based on destination chain
    pub fn new(
        destination_chain_id: u64,
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        match destination_chain_id {
            // Ethereum mainnet and testnets
            1 | 11155111 => {
                let evm_message = EVMCrossChainMessage::new_ethereum(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::EVM(evm_message))
            },
            // BSC mainnet and testnet
            56 | 97 => {
                let evm_message = EVMCrossChainMessage::new_bsc(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::EVM(evm_message))
            },
            // Polygon mainnet and testnet
            137 | 80001 => {
                let evm_message = EVMCrossChainMessage::new_polygon(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::EVM(evm_message))
            },
            // Base mainnet and testnet
            8453 | 84532 => {
                let evm_message = EVMCrossChainMessage::new_base(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::EVM(evm_message))
            },
            // Arbitrum mainnet and testnet
            42161 | 421614 => {
                let evm_message = EVMCrossChainMessage::new_arbitrum(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::EVM(evm_message))
            },
            // Optimism mainnet and testnet
            10 | 11155420 => {
                let evm_message = EVMCrossChainMessage::new_optimism(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::EVM(evm_message))
            },
            // ZetaChain mainnet and testnet
            7000 | 7001 => {
                let zeta_message = ZetaChainCrossChainMessage::new(
                    destination_chain_id, recipient, token_id, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::ZetaChain(zeta_message))
            },
            // Bitcoin mainnet and testnet
            8332 | 18332 => {
                let bitcoin_message = BitcoinCrossChainMessage::new(
                    token_id, recipient, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::Bitcoin(bitcoin_message))
            },
            // Generic format for other chains
            _ => {
                let generic_message = GenericCrossChainMessage::new(
                    destination_chain_id, recipient, token_id, uri, sender, origin_chain, original_mint, is_solana_native
                )?;
                Ok(GatewayCrossChainMessage::Generic(generic_message))
            }
        }
    }

    /// Encode message for gateway transmission
    pub fn encode_for_gateway(&self) -> Result<Vec<u8>> {
        match self {
            GatewayCrossChainMessage::EVM(msg) => msg.encode_borsh(),
            GatewayCrossChainMessage::ZetaChain(msg) => msg.encode_native(),
            GatewayCrossChainMessage::Bitcoin(msg) => msg.encode_bitcoin(),
            GatewayCrossChainMessage::Generic(msg) => msg.encode_borsh(),
        }
    }

    /// Get destination chain ID
    pub fn destination_chain_id(&self) -> u64 {
        match self {
            GatewayCrossChainMessage::EVM(msg) => msg.get_chain_id(),
            GatewayCrossChainMessage::ZetaChain(msg) => msg.destination_chain_id,
            GatewayCrossChainMessage::Bitcoin(_) => 8332, // Bitcoin mainnet
            GatewayCrossChainMessage::Generic(msg) => msg.destination_chain,
        }
    }

    /// Get gas limit for the destination chain
    pub fn gas_limit(&self) -> u64 {
        match self {
            GatewayCrossChainMessage::EVM(_) => 200_000, // Standard EVM gas limit
            GatewayCrossChainMessage::ZetaChain(msg) => msg.destination_gas_limit,
            GatewayCrossChainMessage::Bitcoin(_) => 0, // Bitcoin doesn't use gas
            GatewayCrossChainMessage::Generic(_) => 100_000, // Default gas limit
        }
    }
}

impl EVMCrossChainMessage {
    /// Create EVM message for Ethereum
    pub fn new_ethereum(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let chain_id = 1; // Ethereum mainnet
        let recipient_addr = Self::validate_evm_address(recipient)?;
        let sender_addr = Self::convert_to_evm_address(sender)?;
        let metadata_hash = Self::compute_metadata_hash(&uri);
        let collection_address = Self::derive_collection_address(chain_id, &original_mint.to_bytes());

        Ok(EVMCrossChainMessage {
            token_id,
            recipient: recipient_addr,
            uri,
            sender: sender_addr,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            metadata_hash,
            collection_address,
            chain_id,
        })
    }

    /// Create EVM message for BSC
    pub fn new_bsc(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let chain_id = 56; // BSC mainnet
        let recipient_addr = Self::validate_evm_address(recipient)?;
        let sender_addr = Self::convert_to_evm_address(sender)?;
        let metadata_hash = Self::compute_metadata_hash(&uri);
        let collection_address = Self::derive_collection_address(chain_id, &original_mint.to_bytes());

        Ok(EVMCrossChainMessage {
            token_id,
            recipient: recipient_addr,
            uri,
            sender: sender_addr,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            metadata_hash,
            collection_address,
            chain_id,
        })
    }

    /// Create EVM message for Polygon
    pub fn new_polygon(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let chain_id = 137; // Polygon mainnet
        let recipient_addr = Self::validate_evm_address(recipient)?;
        let sender_addr = Self::convert_to_evm_address(sender)?;
        let metadata_hash = Self::compute_metadata_hash(&uri);
        let collection_address = Self::derive_collection_address(chain_id, &original_mint.to_bytes());

        Ok(EVMCrossChainMessage {
            token_id,
            recipient: recipient_addr,
            uri,
            sender: sender_addr,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            metadata_hash,
            collection_address,
            chain_id,
        })
    }

    /// Create EVM message for Base
    pub fn new_base(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let chain_id = 8453; // Base mainnet
        let recipient_addr = Self::validate_evm_address(recipient)?;
        let sender_addr = Self::convert_to_evm_address(sender)?;
        let metadata_hash = Self::compute_metadata_hash(&uri);
        let collection_address = Self::derive_collection_address(chain_id, &original_mint.to_bytes());

        Ok(EVMCrossChainMessage {
            token_id,
            recipient: recipient_addr,
            uri,
            sender: sender_addr,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            metadata_hash,
            collection_address,
            chain_id,
        })
    }

    /// Create EVM message for Arbitrum
    pub fn new_arbitrum(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let chain_id = 42161; // Arbitrum mainnet
        let recipient_addr = Self::validate_evm_address(recipient)?;
        let sender_addr = Self::convert_to_evm_address(sender)?;
        let metadata_hash = Self::compute_metadata_hash(&uri);
        let collection_address = Self::derive_collection_address(chain_id, &original_mint.to_bytes());

        Ok(EVMCrossChainMessage {
            token_id,
            recipient: recipient_addr,
            uri,
            sender: sender_addr,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            metadata_hash,
            collection_address,
            chain_id,
        })
    }

    /// Create EVM message for Optimism
    pub fn new_optimism(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let chain_id = 10; // Optimism mainnet
        let recipient_addr = Self::validate_evm_address(recipient)?;
        let sender_addr = Self::convert_to_evm_address(sender)?;
        let metadata_hash = Self::compute_metadata_hash(&uri);
        let collection_address = Self::derive_collection_address(chain_id, &original_mint.to_bytes());

        Ok(EVMCrossChainMessage {
            token_id,
            recipient: recipient_addr,
            uri,
            sender: sender_addr,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            metadata_hash,
            collection_address,
            chain_id,
        })
    }

    /// Validate EVM address format
    fn validate_evm_address(address: &[u8]) -> Result<[u8; 20]> {
        if address.len() == 20 {
            let mut addr = [0u8; 20];
            addr.copy_from_slice(address);
            Ok(addr)
        } else if address.len() == 32 {
            // Take last 20 bytes if 32-byte address provided
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&address[12..32]);
            Ok(addr)
        } else {
            Err(UniversalNftError::InvalidRecipientAddress.into())
        }
    }

    /// Convert Solana address to EVM address format
    fn convert_to_evm_address(solana_addr: &[u8]) -> Result<[u8; 20]> {
        if solana_addr.len() == 32 {
            // Hash the Solana address and take first 20 bytes
            let hash = keccak::hash(solana_addr);
            let mut evm_addr = [0u8; 20];
            evm_addr.copy_from_slice(&hash.to_bytes()[0..20]);
            Ok(evm_addr)
        } else if solana_addr.len() == 20 {
            let mut addr = [0u8; 20];
            addr.copy_from_slice(solana_addr);
            Ok(addr)
        } else {
            Err(UniversalNftError::InvalidRecipientAddress.into())
        }
    }

    /// Compute metadata hash for integrity verification
    fn compute_metadata_hash(uri: &str) -> [u8; 32] {
        let hash = keccak::hash(uri.as_bytes());
        hash.to_bytes()
    }

    /// Derive collection address for the destination chain
    fn derive_collection_address(chain_id: u64, collection_pubkey: &[u8]) -> [u8; 20] {
        let mut input = Vec::new();
        input.extend_from_slice(b"universal_nft_collection");
        input.extend_from_slice(&chain_id.to_le_bytes());
        input.extend_from_slice(collection_pubkey);
        let hash = keccak::hash(&input);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash.to_bytes()[0..20]);
        addr
    }

    /// Get chain ID based on message content
    fn get_chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Encode message in proper Ethereum ABI format for EVM chains
    /// Function signature: receiveNFT(uint256,address,string,address,uint256,bytes32,bool,bytes32,address)
    pub fn encode_abi(&self) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        
        // Function selector for receiveNFT(uint256,address,string,address,uint256,bytes32,bool,bytes32,address)
        let function_signature = "receiveNFT(uint256,address,string,address,uint256,bytes32,bool,bytes32,address)";
        let function_hash = keccak::hash(function_signature.as_bytes());
        let function_selector = &function_hash.to_bytes()[0..4];
        encoded.extend_from_slice(function_selector);
        
        // ABI encoding follows head-tail pattern for dynamic types
        // Calculate offsets for dynamic data (string URI)
        let static_data_size = 32 * 8; // 8 parameters * 32 bytes each (heads)
        let uri_offset = static_data_size;
        let uri_bytes = self.uri.as_bytes();
        let uri_padded_len = ((uri_bytes.len() + 31) / 32) * 32; // Round up to 32-byte boundary
        
        // Encode heads (static data and offsets to dynamic data)
        // 1. token_id (uint256) - 32 bytes, big-endian
        let mut token_id_bytes = [0u8; 32];
        token_id_bytes[24..32].copy_from_slice(&self.token_id.to_be_bytes());
        encoded.extend_from_slice(&token_id_bytes);
        
        // 2. recipient (address) - 32 bytes, left-padded to 32 bytes
        let mut recipient_bytes = [0u8; 32];
        recipient_bytes[12..32].copy_from_slice(&self.recipient);
        encoded.extend_from_slice(&recipient_bytes);
        
        // 3. uri (string) - offset to dynamic data
        let mut uri_offset_bytes = [0u8; 32];
        uri_offset_bytes[28..32].copy_from_slice(&(uri_offset as u32).to_be_bytes());
        encoded.extend_from_slice(&uri_offset_bytes);
        
        // 4. sender (address) - 32 bytes, left-padded
        let mut sender_bytes = [0u8; 32];
        sender_bytes[12..32].copy_from_slice(&self.sender);
        encoded.extend_from_slice(&sender_bytes);
        
        // 5. origin_chain (uint256) - 32 bytes, big-endian
        let mut origin_chain_bytes = [0u8; 32];
        origin_chain_bytes[24..32].copy_from_slice(&self.origin_chain.to_be_bytes());
        encoded.extend_from_slice(&origin_chain_bytes);
        
        // 6. original_mint (bytes32) - 32 bytes
        encoded.extend_from_slice(&self.original_mint);
        
        // 7. is_solana_native (bool) - 32 bytes, right-aligned
        let mut bool_bytes = [0u8; 32];
        bool_bytes[31] = if self.is_solana_native { 1 } else { 0 };
        encoded.extend_from_slice(&bool_bytes);
        
        // 8. metadata_hash (bytes32) - 32 bytes
        encoded.extend_from_slice(&self.metadata_hash);
        
        // 9. collection_address (address) - 32 bytes, left-padded
        let mut collection_bytes = [0u8; 32];
        collection_bytes[12..32].copy_from_slice(&self.collection_address);
        encoded.extend_from_slice(&collection_bytes);
        
        // Tail: Dynamic data (URI string)
        // Length of string (32 bytes, big-endian)
        let mut uri_len_bytes = [0u8; 32];
        uri_len_bytes[28..32].copy_from_slice(&(uri_bytes.len() as u32).to_be_bytes());
        encoded.extend_from_slice(&uri_len_bytes);
        
        // String data, padded to 32-byte boundary
        encoded.extend_from_slice(uri_bytes);
        let padding_needed = uri_padded_len - uri_bytes.len();
        if padding_needed > 0 {
            encoded.extend_from_slice(&vec![0u8; padding_needed]);
        }
        
        Ok(encoded)
    }
}

impl ZetaChainCrossChainMessage {
    /// Create ZetaChain message
    pub fn new(
        destination_chain_id: u64,
        recipient: &[u8],
        token_id: u64,
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let destination_address = if recipient.len() == 20 {
            let mut addr = [0u8; 20];
            addr.copy_from_slice(recipient);
            addr
        } else if recipient.len() == 32 {
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&recipient[12..32]);
            addr
        } else {
            return Err(UniversalNftError::InvalidRecipientAddress.into());
        };

        let sender_bytes = if sender.len() == 32 {
            let mut s = [0u8; 32];
            s.copy_from_slice(sender);
            s
        } else {
            return Err(UniversalNftError::InvalidRecipientAddress.into());
        };

        // Create call data for ZetaChain universal app
        let call_data = Self::create_call_data(token_id, &uri, origin_chain, &original_mint.to_bytes(), is_solana_native)?;

        Ok(ZetaChainCrossChainMessage {
            destination_chain_id,
            destination_address,
            destination_gas_limit: 200_000,
            token_id,
            uri,
            sender: sender_bytes,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            call_data,
        })
    }

    /// Create call data for ZetaChain universal app
    fn create_call_data(
        token_id: u64,
        uri: &str,
        origin_chain: u64,
        original_mint: &[u8; 32],
        is_solana_native: bool,
    ) -> Result<Vec<u8>> {
        let mut call_data = Vec::new();
        
        // Method selector for onCrossChainCall
        call_data.extend_from_slice(b"onCrossChainCall");
        
        // Encode parameters
        call_data.extend_from_slice(&token_id.to_le_bytes());
        call_data.extend_from_slice(&(uri.len() as u32).to_le_bytes());
        call_data.extend_from_slice(uri.as_bytes());
        call_data.extend_from_slice(&origin_chain.to_le_bytes());
        call_data.extend_from_slice(original_mint);
        call_data.push(if is_solana_native { 1 } else { 0 });
        
        Ok(call_data)
    }

    /// Encode message in ZetaChain native format
    pub fn encode_native(&self) -> Result<Vec<u8>> {
        self.try_to_vec().map_err(|_| UniversalNftError::InvalidMessage.into())
    }
}

impl BitcoinCrossChainMessage {
    /// Create Bitcoin message
    pub fn new(
        token_id: u64,
        recipient: &[u8],
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        let sender_bytes = if sender.len() == 32 {
            let mut s = [0u8; 32];
            s.copy_from_slice(sender);
            s
        } else {
            return Err(UniversalNftError::InvalidRecipientAddress.into());
        };

        let memo = format!("NFT:{} URI:{} Origin:{}", token_id, uri, origin_chain);

        Ok(BitcoinCrossChainMessage {
            token_id,
            recipient: recipient.to_vec(),
            uri,
            sender: sender_bytes,
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            memo,
        })
    }

    /// Encode message for Bitcoin
    pub fn encode_bitcoin(&self) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        
        // Bitcoin OP_RETURN format
        encoded.push(0x6a); // OP_RETURN
        
        let data = format!(
            "NFT:{}:{}:{}:{}",
            self.token_id,
            self.uri,
            self.origin_chain,
            if self.is_solana_native { "SOL" } else { "EXT" }
        );
        
        let data_bytes = data.as_bytes();
        encoded.push(data_bytes.len() as u8);
        encoded.extend_from_slice(data_bytes);
        
        Ok(encoded)
    }
}

impl GenericCrossChainMessage {
    /// Create generic message
    pub fn new(
        destination_chain: u64,
        recipient: &[u8],
        token_id: u64,
        uri: String,
        sender: &[u8],
        origin_chain: u64,
        original_mint: Pubkey,
        is_solana_native: bool,
    ) -> Result<Self> {
        Ok(GenericCrossChainMessage {
            destination_chain,
            recipient: recipient.to_vec(),
            token_id,
            uri,
            sender: sender.to_vec(),
            origin_chain,
            original_mint: original_mint.to_bytes(),
            is_solana_native,
            custom_data: Vec::new(),
        })
    }

    /// Encode message using Borsh
    pub fn encode_borsh(&self) -> Result<Vec<u8>> {
        self.try_to_vec().map_err(|_| UniversalNftError::InvalidMessage.into())
    }
}

/// Gateway PDA management functions
pub fn find_gateway_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[GATEWAY_PDA_SEED], program_id)
}


/// Create gateway deposit instruction
pub fn create_gateway_deposit_instruction(
    gateway_program_id: &Pubkey,
    gateway_pda: &Pubkey,
    sender: &Pubkey,
    amount: u64,
    receiver: [u8; 20],
    revert_options: Option<RevertOptions>,
) -> Result<Instruction> {
    let instruction_data = ZetaGatewayInstruction::Deposit {
        amount,
        receiver,
        revert_options,
    };

    let data = instruction_data.try_to_vec()
        .map_err(|_| UniversalNftError::InvalidMessage)?;

    Ok(Instruction {
        program_id: *gateway_program_id,
        accounts: vec![
            AccountMeta::new(*gateway_pda, false),
            AccountMeta::new_readonly(*gateway_program_id, false),
            AccountMeta::new(*sender, true),
            AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
            AccountMeta::new_readonly(anchor_lang::solana_program::sysvar::rent::ID, false),
        ],
        data,
    })
}

/// Create gateway deposit and call instruction
pub fn create_gateway_deposit_and_call_instruction(
    gateway_program_id: &Pubkey,
    gateway_pda: &Pubkey,
    sender: &Pubkey,
    amount: u64,
    receiver: [u8; 20],
    message: Vec<u8>,
    revert_options: Option<RevertOptions>,
) -> Result<Instruction> {
    let instruction_data = ZetaGatewayInstruction::DepositAndCall {
        amount,
        receiver,
        message,
        revert_options,
    };

    let data = instruction_data.try_to_vec()
        .map_err(|_| UniversalNftError::InvalidMessage)?;

    Ok(Instruction {
        program_id: *gateway_program_id,
        accounts: vec![
            AccountMeta::new(*gateway_pda, false),
            AccountMeta::new(*sender, true),
            AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
            AccountMeta::new_readonly(anchor_lang::solana_program::sysvar::rent::ID, false),
        ],
        data,
    })
}

/// Create gateway call instruction
pub fn create_gateway_call_instruction(
    gateway_program_id: &Pubkey,
    gateway_pda: &Pubkey,
    sender: &Pubkey,
    receiver: [u8; 20],
    message: Vec<u8>,
    call_options: CallOptions,
) -> Result<Instruction> {
    let instruction_data = ZetaGatewayInstruction::Call {
        receiver,
        message,
        call_options,
    };

    let data = instruction_data.try_to_vec()
        .map_err(|_| UniversalNftError::InvalidMessage)?;

    Ok(Instruction {
        program_id: *gateway_program_id,
        accounts: vec![
            AccountMeta::new(*gateway_pda, false),
            AccountMeta::new(*sender, true),
            AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
        ],
        data,
    })
}

/// Execute gateway CPI call
pub fn execute_gateway_cpi<'info>(
    instruction: &Instruction,
    account_infos: &[AccountInfo<'info>],
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    if signer_seeds.is_empty() {
        anchor_lang::solana_program::program::invoke(instruction, account_infos)
            .map_err(|_| UniversalNftError::UnauthorizedGateway)?;
    } else {
        invoke_signed(instruction, account_infos, signer_seeds)
            .map_err(|_| UniversalNftError::UnauthorizedGateway)?;
    }
    Ok(())
}

/// Validate gateway PDA
pub fn validate_gateway_pda(
    gateway_pda: &Pubkey,
    expected_program_id: &Pubkey,
) -> Result<u8> {
    let (expected_pda, bump) = find_gateway_pda(expected_program_id);
    require!(
        *gateway_pda == expected_pda,
        UniversalNftError::UnauthorizedGateway
    );
    Ok(bump)
}

/// Create revert options for failed transfers
pub fn create_revert_options(
    revert_address: [u8; 20],
    call_on_revert: bool,
    abort_address: [u8; 20],
    revert_message: Vec<u8>,
    on_revert_gas_limit: u64,
) -> RevertOptions {
    RevertOptions {
        revert_address,
        call_on_revert,
        abort_address,
        revert_message,
        on_revert_gas_limit,
    }
}

/// Decode incoming gateway message
pub fn decode_gateway_message(message: &[u8], source_chain_id: u64) -> Result<CrossChainMessage> {
    match source_chain_id {
        // EVM chains
        1 | 11155111 | 56 | 97 | 137 | 80001 | 8453 | 84532 | 42161 | 421614 | 10 | 11155420 => {
            let evm_message = EVMCrossChainMessage::try_from_slice(message)
                .map_err(|_| UniversalNftError::InvalidMessage)?;
            Ok(CrossChainMessage::EVM(evm_message))
        },
        // ZetaChain
        7000 | 7001 => {
            let zeta_message = ZetaChainCrossChainMessage::try_from_slice(message)
                .map_err(|_| UniversalNftError::InvalidMessage)?;
            Ok(CrossChainMessage::ZetaChain(zeta_message))
        },
        // Bitcoin
        8332 | 18332 => {
            let bitcoin_message = BitcoinCrossChainMessage::try_from_slice(message)
                .map_err(|_| UniversalNftError::InvalidMessage)?;
            Ok(CrossChainMessage::Bitcoin(bitcoin_message))
        },
        // Generic
        _ => {
            let generic_message = GenericCrossChainMessage::try_from_slice(message)
                .map_err(|_| UniversalNftError::InvalidMessage)?;
            Ok(CrossChainMessage::Generic(generic_message))
        }
    }
}

/// Validate cross-chain message integrity
pub fn validate_message_integrity(
    message: &CrossChainMessage,
    expected_token_id: u64,
    expected_origin_chain: u64,
) -> Result<()> {
    let (token_id, origin_chain) = match message {
        CrossChainMessage::EVM(msg) => (msg.token_id, msg.origin_chain),
        CrossChainMessage::ZetaChain(msg) => (msg.token_id, msg.origin_chain),
        CrossChainMessage::Bitcoin(msg) => (msg.token_id, msg.origin_chain),
        CrossChainMessage::Generic(msg) => (msg.token_id, msg.origin_chain),
    };

    require!(
        token_id == expected_token_id,
        UniversalNftError::InvalidTokenId
    );

    require!(
        origin_chain == expected_origin_chain,
        UniversalNftError::InvalidMessage
    );

    Ok(())
}

/// Get supported destination chains
pub fn get_supported_destination_chains() -> Vec<u64> {
    vec![
        1,      // Ethereum mainnet
        11155111, // Ethereum Sepolia
        56,     // BSC mainnet
        97,     // BSC testnet
        137,    // Polygon mainnet
        80001,  // Polygon Mumbai
        8453,   // Base mainnet
        84532,  // Base Sepolia
        42161,  // Arbitrum mainnet
        421614, // Arbitrum Sepolia
        10,     // Optimism mainnet
        11155420, // Optimism Sepolia
        7000,   // ZetaChain mainnet
        7001,   // ZetaChain testnet
        8332,   // Bitcoin mainnet
        18332,  // Bitcoin testnet
    ]
}

/// Check if destination chain is supported
pub fn is_destination_chain_supported(chain_id: u64) -> bool {
    get_supported_destination_chains().contains(&chain_id)
}

/// Get chain name from chain ID
pub fn get_chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "Ethereum Mainnet",
        11155111 => "Ethereum Sepolia",
        56 => "BSC Mainnet",
        97 => "BSC Testnet",
        137 => "Polygon Mainnet",
        80001 => "Polygon Mumbai",
        8453 => "Base Mainnet",
        84532 => "Base Sepolia",
        42161 => "Arbitrum Mainnet",
        421614 => "Arbitrum Sepolia",
        10 => "Optimism Mainnet",
        11155420 => "Optimism Sepolia",
        7000 => "ZetaChain Mainnet",
        7001 => "ZetaChain Testnet",
        8332 => "Bitcoin Mainnet",
        18332 => "Bitcoin Testnet",
        103 => "Solana Devnet",
        102 => "Solana Testnet",
        101 => "Solana Mainnet",
        _ => "Unknown Chain",
    }
}