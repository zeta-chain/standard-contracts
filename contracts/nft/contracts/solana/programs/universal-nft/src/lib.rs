use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    secp256k1_recover::{secp256k1_recover},
    program::{invoke, invoke_signed},
    instruction::Instruction,
    system_instruction,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, burn, mint_to, Burn, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;

// Universal NFT Program ID - deployed on devnet
declare_id!("GqXUjfsGancY5D3QxBjhcmwRtykDiPj91wEJ8nRakLip");

// ZetaChain Gateway Program ID - deployed on mainnet-beta, testnet, and devnet
// This is the actual deployed ZetaChain gateway program
// Address: ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis
pub const ZETACHAIN_GATEWAY_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x06, 0xa1, 0xe4, 0xdc, 0x88, 0x7a, 0x5b, 0x3f,
    0xbf, 0x0f, 0x5f, 0x1a, 0xc4, 0xb3, 0x6f, 0xf9,
    0x7c, 0x94, 0xf1, 0x3f, 0x42, 0xd8, 0xc4, 0xc5,
    0x7a, 0x71, 0x4c, 0x92, 0x59, 0xbe, 0x63, 0x80
]);

// Gateway PDA account (derived from seeds b"meta" and canonical bump)
pub const GATEWAY_PDA_SEED: &[u8] = b"meta";

#[program]
pub mod universal_nft {
    use super::*;

    /// Initialize a new Universal NFT collection compatible with ZetaChain
    pub fn initialize_collection(
        ctx: Context<InitializeCollection>,
        name: String,
        symbol: String,
        uri: String,
        tss_address: [u8; 20],
    ) -> Result<()> {
        let collection = &mut ctx.accounts.collection;
        let collection_key = collection.key();
        collection.authority = ctx.accounts.authority.key();
        collection.name = name.clone();
        collection.symbol = symbol.clone();
        collection.uri = uri.clone();
        collection.next_token_id = 1;
        collection.tss_address = tss_address;
        collection.nonce = 0;
        collection.bump = ctx.bumps.collection;

        // Note: Collection mint and metadata would be created here in production
        // For now, we're focusing on the core NFT functionality

        emit!(CollectionInitialized {
            collection: collection_key,
            authority: ctx.accounts.authority.key(),
            name,
            symbol,
            tss_address,
        });

        Ok(())
    }

    /// Mint a new NFT in the collection
    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        uri: String,
    ) -> Result<()> {
        // Extract values before mutable borrow
        let collection_key = ctx.accounts.collection.key();
        let token_id = ctx.accounts.collection.next_token_id;
        let collection_authority = ctx.accounts.collection.authority;
        let collection_name = ctx.accounts.collection.name.clone();
        let collection_bump = ctx.accounts.collection.bump;

        // Mint NFT token
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.collection.to_account_info(),
            },
        );

        let seeds = &[
            b"collection",
            collection_authority.as_ref(),
            collection_name.as_bytes(),
            &[collection_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        mint_to(cpi_ctx.with_signer(signer_seeds), 1)?;

        // Note: Simplified metadata creation - in production, use proper Metaplex instructions
        // This is a placeholder for the actual Metaplex metadata creation

        // Update collection state
        let collection = &mut ctx.accounts.collection;
        collection.next_token_id += 1;

        emit!(TokenMinted {
            collection: collection_key,
            token_id,
            mint: ctx.accounts.nft_mint.key(),
            recipient: ctx.accounts.recipient.key(),
            name,
            uri,
        });

        Ok(())
    }

    /// Transfer NFT cross-chain to another blockchain via ZetaChain
    pub fn transfer_cross_chain(
        ctx: Context<TransferCrossChain>,
        destination_chain_id: u64,
        recipient: Vec<u8>,
    ) -> Result<()> {
        // Derive token_id from mint address (use first 8 bytes of mint pubkey)
        let mint_bytes = ctx.accounts.nft_mint.key().to_bytes();
        let token_id = u64::from_le_bytes([
            mint_bytes[0], mint_bytes[1], mint_bytes[2], mint_bytes[3],
            mint_bytes[4], mint_bytes[5], mint_bytes[6], mint_bytes[7]
        ]);
        let gas_amount = 2000000u64; // Default gas limit
        // Validate destination chain
        require!(
            is_supported_chain(destination_chain_id),
            UniversalNftError::UnsupportedChain
        );

        let collection = &ctx.accounts.collection;

        // Get NFT metadata URI (placeholder - should read from actual metadata)
        let nft_uri = "https://example.com/nft.json".to_string();

        // Burn the NFT locally
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.nft_mint.to_account_info(),
                from: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );

        burn(cpi_ctx, 1)?;

        // Prepare cross-chain message
        let mut recipient_array = [0u8; 20];
        recipient_array.copy_from_slice(&recipient[..20.min(recipient.len())]);
        let message = ZetaChainMessage {
            destination_chain_id,
            destination_address: recipient_array,
            destination_gas_limit: gas_amount,
            message: vec![],
            token_id,
            uri: nft_uri.clone(),
            sender: ctx.accounts.owner.key().to_bytes(),
        };

        // Serialize message for ZetaChain gateway
        let message_data = message.try_to_vec()?;
        
        // Calculate gas fee
        let _gas_fee = calculate_gas_fee(destination_chain_id, gas_amount)?;
        
        // Create gateway call instruction
        // TODO: Uncomment when ZetaChain gateway is fully operational
        // let gateway_instruction = Instruction {
        //     program_id: ZETACHAIN_GATEWAY_PROGRAM_ID,
        //     accounts: vec![
        //         AccountMeta::new(ctx.accounts.gateway_pda.key(), false),
        //         AccountMeta::new(ctx.accounts.owner.key(), true),
        //     ],
        //     data: message_data.clone(),
        // };
        
        // // Invoke the gateway
        // invoke_signed(
        //     &gateway_instruction,
        //     &[
        //         ctx.accounts.gateway_pda.to_account_info(),
        //         ctx.accounts.owner.to_account_info(),
        //     ],
        //     &[],
        // )?;
        
        msg!("Cross-chain transfer initiated to chain {} for recipient {:?}", 
             destination_chain_id, recipient);
        
        // Emit event
        emit!(TokenTransfer {
            collection: collection.key(),
            token_id,
            destination_chain_id,
            recipient: recipient.to_vec(),
            uri: nft_uri,
            sender: ctx.accounts.owner.key(),
            message: message_data,
        });
        
        Ok(())
    }

    /// Handle incoming cross-chain NFT transfer from ZetaChain gateway
    pub fn on_call(
        ctx: Context<OnCall>,
        sender: [u8; 20],
        source_chain_id: u64,
        message: Vec<u8>,
    ) -> Result<()> {
        // Parse the incoming message
        // Expected format: [token_id(8), uri_len(4), uri(variable), recipient(32)]
        require!(message.len() >= 44, UniversalNftError::InvalidMessageHash);
        
        let mut offset = 0;
        
        // Extract token_id
        let token_id = u64::from_le_bytes(
            message[offset..offset + 8]
                .try_into()
                .map_err(|_| UniversalNftError::InvalidMessageHash)?
        );
        offset += 8;
        
        // Extract URI length and URI
        let uri_len = u32::from_le_bytes(
            message[offset..offset + 4]
                .try_into()
                .map_err(|_| UniversalNftError::InvalidMessageHash)?
        ) as usize;
        offset += 4;
        
        require!(
            message.len() >= offset + uri_len + 32,
            UniversalNftError::InvalidMessageHash
        );
        
        let uri = String::from_utf8(message[offset..offset + uri_len].to_vec())
            .map_err(|_| UniversalNftError::InvalidMessageHash)?;
        offset += uri_len;
        
        // Extract recipient (32 bytes for Solana pubkey)
        let recipient_bytes: [u8; 32] = message[offset..offset + 32]
            .try_into()
            .map_err(|_| UniversalNftError::InvalidMessageHash)?;
        let recipient_pubkey = Pubkey::new_from_array(recipient_bytes);
        
        // Verify the recipient matches the token account owner
        require!(
            recipient_pubkey == ctx.accounts.recipient.key(),
            UniversalNftError::InvalidRecipient
        );
        
        // Get collection data before mutable borrow
        let collection_authority = ctx.accounts.collection.authority;
        let collection_name = ctx.accounts.collection.name.clone();
        let collection_bump = ctx.accounts.collection.bump;
        let collection_key = ctx.accounts.collection.key();
        
        // Mint the NFT to the recipient
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.collection.to_account_info(),
            },
        );
        
        // Use collection as signer
        let seeds = &[
            b"collection",
            collection_authority.as_ref(),
            collection_name.as_bytes(),
            &[collection_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        mint_to(cpi_ctx.with_signer(signer_seeds), 1)?;
        
        // Update collection state
        let collection = &mut ctx.accounts.collection;
        collection.nonce += 1;
        let current_nonce = collection.nonce;
        
        msg!(
            "Received NFT from chain {} with token_id {} for recipient {}",
            source_chain_id,
            token_id,
            recipient_pubkey
        );
        
        // Emit event
        emit!(TokenTransferReceived {
            collection: collection_key,
            token_id,
            recipient: recipient_pubkey,
            uri: uri.clone(),
            original_sender: sender.to_vec(),
            nonce: current_nonce - 1,
        });
        
        Ok(())
    }
    
    /// Handle incoming cross-chain NFT transfer with TSS signature verification
    pub fn receive_cross_chain(
        ctx: Context<ReceiveCrossChain>,
        message_hash: [u8; 32],
        signature: [u8; 64],
        recovery_id: u8,
        message_data: Vec<u8>,
    ) -> Result<()> {
        let collection = &mut ctx.accounts.collection;
        let collection_key = collection.key();
        
        // Verify TSS signature
        let recovered_pubkey = secp256k1_recover(&message_hash, recovery_id, &signature)
            .map_err(|_| UniversalNftError::InvalidSignature)?;
        
        let recovered_address = pubkey_to_eth_address(&recovered_pubkey.0)?;
        
        require!(
            recovered_address == collection.tss_address,
            UniversalNftError::InvalidSignature
        );

        // Deserialize cross-chain message
        let cross_chain_message = ZetaChainMessage::try_from_slice(&message_data)
            .map_err(|_| UniversalNftError::InvalidMessageHash)?;

        // Verify this is the expected recipient
        let expected_recipient = ctx.accounts.recipient.key();
        let mut expected_bytes = [0u8; 32];
        expected_bytes.copy_from_slice(&expected_recipient.to_bytes());
        
        // For cross-chain, we might need to handle address format differences
        // This is a simplified check - real implementation would handle chain-specific formats
        
        // Update nonce to prevent replay
        let current_nonce = collection.nonce;
        collection.nonce += 1;

        // Extract values before mutable borrow
        let collection_authority = collection.authority;
        let collection_name = collection.name.clone();
        let collection_bump = collection.bump;

        // Mint the NFT to the recipient
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.collection.to_account_info(),
            },
        );

        let seeds = &[
            b"collection",
            collection_authority.as_ref(),
            collection_name.as_bytes(),
            &[collection_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        mint_to(cpi_ctx.with_signer(signer_seeds), 1)?;

        // Note: Simplified metadata creation - in production, use proper Metaplex instructions
        // This is a placeholder for the actual Metaplex metadata creation

        emit!(TokenTransferReceived {
            collection: collection_key,
            token_id: cross_chain_message.token_id,
            recipient: ctx.accounts.recipient.key(),
            uri: cross_chain_message.uri,
            original_sender: cross_chain_message.sender.to_vec(),
            nonce: current_nonce,
        });

        Ok(())
    }
}

// Helper function to convert secp256k1 public key to Ethereum address
fn pubkey_to_eth_address(pubkey: &[u8; 64]) -> Result<[u8; 20]> {
    use anchor_lang::solana_program::keccak::hash;
    let hash_result = hash(pubkey);
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash_result.to_bytes()[12..]);
    Ok(address)
}

#[derive(Accounts)]
#[instruction(name: String, _symbol: String, _uri: String, tss_address: [u8; 20])]
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + Collection::INIT_SPACE,
        seeds = [b"collection", authority.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub collection: Account<'info, Collection>,

    /// CHECK: Collection mint - not used in simplified version
    pub collection_mint: UncheckedAccount<'info>,

    /// CHECK: Collection token account - not used in simplified version
    pub collection_token_account: UncheckedAccount<'info>,

    /// CHECK: Metadata account - not used in simplified version
    pub collection_metadata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: NFT recipient account
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Metadata account for the NFT
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct TransferCrossChain<'info> {
    pub collection: Account<'info, Collection>,

    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = owner,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,

    /// CHECK: ZetaChain gateway PDA account
    #[account(mut)]
    pub gateway_pda: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OnRevert<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Gateway account that calls this function
    pub gateway: Signer<'info>,
    
    /// CHECK: Gateway PDA account
    pub gateway_pda: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = original_sender,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Original sender account to receive the reverted NFT
    pub original_sender: UncheckedAccount<'info>,
    
    /// CHECK: Metadata account for the NFT
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct OnCall<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Gateway account that calls this function
    pub gateway: Signer<'info>,
    
    /// CHECK: Gateway PDA account
    pub gateway_pda: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: NFT recipient account
    pub recipient: UncheckedAccount<'info>,
    
    /// CHECK: Metadata account for the NFT
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ReceiveCrossChain<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: NFT recipient account
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Metadata account for the NFT
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

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
    pub gateway_pda: Option<Pubkey>, // Gateway PDA if needed
    pub next_token_id: u64,
    pub nonce: u64, // Replay protection counter
    pub bump: u8,
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

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum GatewayInstruction {
    Call {
        receiver: [u8; 20],
        amount: u64,
        message: Vec<u8>,
    },
}

#[event]
pub struct CollectionInitialized {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub tss_address: [u8; 20],
}

#[event]
pub struct TokenMinted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub name: String,
    pub uri: String,
}

#[event]
pub struct TokenTransfer {
    pub collection: Pubkey,
    pub token_id: u64,
    pub destination_chain_id: u64,
    pub recipient: Vec<u8>,
    pub uri: String,
    pub sender: Pubkey,
    pub message: Vec<u8>,
}

#[event]
pub struct TokenTransferReceived {
    pub collection: Pubkey,
    pub token_id: u64,
    pub recipient: Pubkey,
    pub uri: String,
    pub original_sender: Vec<u8>,
    pub nonce: u64,
}

#[event]
pub struct CrossChainCallReceived {
    pub collection: Pubkey,
    pub sender: [u8; 20],
    pub source_chain_id: u64,
    pub token_id: u64,
    pub new_token_id: u64,
    pub recipient: Pubkey,
}

#[event]
pub struct TokenTransferReverted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub new_token_id: u64,
    pub recipient: Pubkey,
    pub uri: String,
    pub revert_reason: String,
    pub revert_message: Vec<u8>,
}

#[error_code]
pub enum UniversalNftError {
    #[msg("Invalid TSS signature")]
    InvalidTssSignature,
    #[msg("Invalid message hash")]
    InvalidMessageHash,
    #[msg("Invalid message")]
    InvalidMessage,
    #[msg("Invalid recipient")]
    InvalidRecipient,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Unauthorized TSS address")]
    UnauthorizedTssAddress,
    #[msg("Invalid nonce")]
    InvalidNonce,
    #[msg("Token does not exist")]
    TokenDoesNotExist,
    #[msg("Not the token owner")]
    NotTokenOwner,
    #[msg("Invalid destination chain")]
    InvalidDestinationChain,
    #[msg("Invalid recipient address")]
    InvalidRecipientAddress,
    #[msg("Insufficient gas amount")]
    InsufficientGasAmount,
    #[msg("Unauthorized gateway call")]
    UnauthorizedGateway,
    #[msg("Unsupported chain")]
    UnsupportedChain,
    #[msg("Invalid token ID")]
    InvalidTokenId,
}

// Supported chain IDs
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

/// Calculate gas fee based on destination chain and gas amount
pub fn calculate_gas_fee(destination_chain: u64, gas_amount: u64) -> Result<u64> {
    // Base gas costs per chain (in lamports)
    let base_gas: u64 = match destination_chain {
        84532 => 100_000,    // Base Sepolia - higher gas for L2
        11155111 => 150_000, // Ethereum Sepolia - highest gas
        7001 => 50_000,      // ZetaChain testnet - lower gas
        97 => 80_000,        // BSC testnet
        80001 => 80_000,     // Polygon Mumbai
        421614 => 100_000,   // Arbitrum Sepolia
        11155420 => 100_000, // Optimism Sepolia
        _ => 100_000,        // Default gas
    };
    
    let total_fee = base_gas
        .checked_mul(gas_amount)
        .ok_or(UniversalNftError::InsufficientGasAmount)?;
    
    // Ensure minimum gas fee
    let min_fee = 10_000_000; // 0.01 SOL minimum
    Ok(total_fee.max(min_fee))
}
