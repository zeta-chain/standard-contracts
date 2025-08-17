use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_spl::token::{self, MintTo, Burn};

declare_id!("BKfF2J9U6Umt7mkiqZXHF5RPpmTgWdKHH6MzgRQp9G36");

// Module declarations
pub mod error;
pub mod util;
pub mod state;
pub mod event;
pub mod context;

use error::UniversalNftError;
use util::constants::*;
use util::{create_metadata_account, create_master_edition_account, decode_nft_data_with_recipient};
use util::mint_helpers::mint_nft_to_recipient;
use state::*;
use event::*;
use context::*;

#[program]
pub mod universal_nft {
    use super::*;

    /// Initialize the Universal NFT program
    /// Sets up global configuration and authority
    pub fn initialize(
        ctx: Context<Initialize>,
        gateway_program: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let clock = Clock::get()?;
        
        config.authority = ctx.accounts.authority.key();
        config.gateway_program = gateway_program;
        config.nonce = 0;
        config.is_paused = false;
        config.created_at = clock.unix_timestamp;
        config.bump = ctx.bumps.config;
        
        emit!(ProgramConfigured {
            authority: config.authority,
            gateway_program: config.gateway_program,
            timestamp: config.created_at,
        });
        
        msg!("Universal NFT program initialized by: {:?}", config.authority);
        Ok(())
    }

    /// Mint a new Universal NFT on Solana
    /// Creates NFT, metadata, master edition, and origin tracking
    /// 
    /// This function is used for:
    /// 1. Minting brand new NFTs on Solana (original minting)
    /// 2. Minting NFTs from connected chains (when no origin PDA exists)
    /// 
    /// For returning Solana NFTs (when origin PDA exists), use restore_returning_nft() instead
    pub fn mint_nft(
        ctx: Context<MintNft>,
        token_id: [u8; 32],
        name: String,
        symbol: String,
        uri: String
    ) -> Result<()> {
        // Validate inputs
        require!(name.len() <= MAX_NAME_LENGTH, UniversalNftError::NameTooLong);
        require!(symbol.len() <= MAX_SYMBOL_LENGTH, UniversalNftError::SymbolTooLong);
        require!(uri.len() <= MAX_URI_LENGTH, UniversalNftError::UriTooLong);
        require!(!ctx.accounts.config.is_paused, UniversalNftError::OperationNotAllowed);
        
        let clock = Clock::get()?;
        
        // Create metadata account using CPI to Metaplex Token Metadata program
        create_metadata_account(
            &ctx.accounts.metadata.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.authority.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &name,
            &symbol,
            &uri,
        )?;
        
        // Create master edition account
        create_master_edition_account(
            &ctx.accounts.master_edition.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.authority.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.metadata.to_account_info(),
            &ctx.accounts.metadata_program.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
        )?;
        
        // Mint NFT to recipient
        let mint_to_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );
        token::mint_to(mint_to_ctx, 1)?;
        
        // Set up origin tracking
        let nft_origin = &mut ctx.accounts.nft_origin;
        **nft_origin = NftOrigin::new(
            token_id,
            ctx.accounts.mint.key(),
            ctx.accounts.metadata.key(),
            uri.clone(),
            clock.unix_timestamp,
            ctx.bumps.nft_origin,
        );

        emit!(NftMinted {
            mint: ctx.accounts.mint.key(),
            token_id,
            owner: ctx.accounts.recipient.key(),
            uri: uri.clone(),
            timestamp: clock.unix_timestamp
        });
        
        emit!(OriginTracked {
            token_id,
            original_mint: ctx.accounts.mint.key(),
            original_metadata: ctx.accounts.metadata.key(),
            original_uri: uri,
            created_at: clock.unix_timestamp,
            timestamp: clock.unix_timestamp,
        });
        
        msg!("NFT minted successfully with token_id: {:?}", token_id);
        Ok(())
    }

    /// Transfer NFT to ZetaChain via deposit_and_call
    /// Burns the NFT on Solana and deposits to universal contract on ZetaChain
    pub fn transfer_to_zetachain(
        ctx: Context<TransferToZetachain>,
        token_id: [u8; 32],
        zetachain_universal_contract: [u8; 20], // ZetaChain universal NFT contract address
        final_destination_chain: u64, // If 0, stays on ZetaChain
        final_recipient: String, // Final recipient on destination chain
    ) -> Result<()> {
        require!(!ctx.accounts.config.is_paused, UniversalNftError::OperationNotAllowed);
        require!(!final_recipient.is_empty(), UniversalNftError::InvalidRecipientAddress);
        
        let clock = Clock::get()?;
        
        // Burn the NFT on Solana
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::burn(burn_ctx, 1)?;
        
        // Update origin tracking
        let nft_origin = &mut ctx.accounts.nft_origin;
        nft_origin.mark_transferred_off_solana(clock.unix_timestamp);
        
        // Create message for ZetaChain universal contract
        // For authenticated calls (which go to onCall), we only need ABI-encoded arguments
        // No function selector is required as per ZetaChain documentation
        let cross_chain_message = {
            let mut message = Vec::new();
            // Token ID (32 bytes)
            message.extend_from_slice(&token_id);
            // Source chain (Solana) (8 bytes)
            message.extend_from_slice(&SOLANA_CHAIN_ID.to_be_bytes());
            // Metadata URI length (4 bytes) + URI data
            let uri_bytes = nft_origin.original_uri.as_bytes();
            message.extend_from_slice(&(uri_bytes.len() as u32).to_be_bytes());
            message.extend_from_slice(uri_bytes);
            // Final destination chain (8 bytes)
            message.extend_from_slice(&final_destination_chain.to_be_bytes());
            // Final recipient length (4 bytes) + recipient data
            let recipient_bytes = final_recipient.as_bytes();
            message.extend_from_slice(&(recipient_bytes.len() as u32).to_be_bytes());
            message.extend_from_slice(recipient_bytes);
            message
        };
        
        // Call ZetaChain gateway deposit_and_call via CPI
        //
        // According to ZetaChain documentation (https://www.zetachain.com/docs/developers/chains/solana/)
        // "For universal apps (contracts that can be called from other chains), the application must use
        // ZetaChain's gateway contract to initiate cross-chain calls. The gateway ensures proper message
        // routing and delivers the call to the onCall handler on the destination universal contract."
        //
        // The deposit_and_call function specifically:
        // 1. Deposits SOL to the gateway PDA 
        // 2. Encodes the cross-chain message with NFT data
        // 3. Triggers ZetaChain's cross-chain infrastructure to deliver the message
        // 4. Results in the onCall handler being executed on the destination chain
        //
        // Without this gateway call, there would be no cross-chain transfer mechanism.
        msg!("Initiating cross-chain NFT transfer via ZetaChain gateway");
        // This makes a cross-program invocation to the ZetaChain gateway program
        // which handles depositing SOL and calling the universal contract on ZetaChain
        let gateway_program_info = ctx.accounts.gateway_program.to_account_info();
        let gateway_pda_info = ctx.accounts.gateway_pda.to_account_info();
        let owner_info = ctx.accounts.owner.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();
        
        // Encode the receiver (ZetaChain universal contract) as bytes
        let receiver_bytes = zetachain_universal_contract.to_vec();
        
        // Create CPI instruction to ZetaChain gateway
        let deposit_and_call_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.config.gateway_program,
            accounts: vec![
                AccountMeta::new(gateway_pda_info.key(), false), // Gateway PDA
                AccountMeta::new(owner_info.key(), true),       // Signer (owner)
                AccountMeta::new_readonly(system_program_info.key(), false), // System Program
            ],
            data: {
                // Instruction data format for gateway depositAndCall:
                // [instruction_discriminator(8)] + [amount(8)] + [receiver_len(4)] + [receiver] + [message_len(4)] + [message]
                let mut instruction_data = Vec::new();
                
                // Instruction discriminator for depositAndCall (SHA256("global:deposit_and_call")[:8])
                // This matches the gateway program's deposit_and_call method exactly
                instruction_data.extend_from_slice(&[65, 33, 186, 198, 114, 223, 133, 57]);
                
                // Amount (0 SOL for NFT transfer, just paying for gas)
                instruction_data.extend_from_slice(&0u64.to_le_bytes());
                
                // Receiver length and data
                instruction_data.extend_from_slice(&(receiver_bytes.len() as u32).to_le_bytes());
                instruction_data.extend_from_slice(&receiver_bytes);
                
                // Message length and data
                instruction_data.extend_from_slice(&(cross_chain_message.len() as u32).to_le_bytes());
                instruction_data.extend_from_slice(&cross_chain_message);
                
                instruction_data
            },
        };
        
        // Execute CPI call to ZetaChain gateway
        anchor_lang::solana_program::program::invoke(
            &deposit_and_call_ix,
            &[
                gateway_pda_info,
                owner_info,
                system_program_info,
            ],
        )?;
        
        let config = &mut ctx.accounts.config;
        config.nonce = config.nonce.checked_add(1).ok_or(UniversalNftError::ArithmeticOverflow)?;
        
        emit!(CrossChainTransferInitiated {
            token_id,
            source_chain: SOLANA_CHAIN_ID,
            destination_chain: final_destination_chain,
            sender: ctx.accounts.owner.key(),
            recipient: final_recipient.clone(),
            metadata_uri: nft_origin.original_uri.clone(),
            gas_limit: 0, // Not needed for Solana -> ZetaChain
            timestamp: clock.unix_timestamp,
            nonce: config.nonce,
        });
        
        emit!(NftBurned {
            mint: ctx.accounts.mint.key(),
            token_id,
            owner: ctx.accounts.owner.key(),
            timestamp: clock.unix_timestamp,
            reason: "transfer_to_zetachain".to_string(),
        });
        
        msg!("NFT transferred to ZetaChain universal contract: {:?}", zetachain_universal_contract);
        Ok(())
    }

    pub fn on_call(
        ctx: Context<OnCall>,
        amount: u64, // Amount of SOL sent (if any)
        sender: [u8; 20], // ZetaChain universal contract address that initiated the call
        data: Vec<u8>, // Encoded NFT data (token_id, recipient, metadata_uri, name, symbol, etc.)
    ) -> Result<()> {
        require!(!ctx.accounts.config.is_paused, UniversalNftError::OperationNotAllowed);
        
        // Verify that the call is coming from ZetaChain gateway
        let current_ix = anchor_lang::solana_program::sysvar::instructions::get_instruction_relative(
            0,
            &ctx.accounts.instruction_sysvar_account.to_account_info(),
        ).map_err(|_| UniversalNftError::InvalidCaller)?;
        
        require!(
            current_ix.program_id == ctx.accounts.config.gateway_program,
            UniversalNftError::InvalidCaller
        );
        
        // Decode the cross-chain message data to get token_id and other info
        let (token_id, source_chain, recipient_bytes, metadata_uri, name, symbol) = decode_nft_data_with_recipient(&data)?;
        
        // Convert recipient bytes to Pubkey
        let recipient = Pubkey::try_from(recipient_bytes)
            .map_err(|_| UniversalNftError::InvalidRecipientAddress)?;
        
        msg!(
            "Cross-chain NFT received from chain {} with token_id: {:?}, recipient: {}, metadata: {}", 
            source_chain, 
            token_id, 
            recipient,
            metadata_uri
        );
        
        Ok(())
    }

    /// Restore a returning Solana NFT (when NFT origin PDA exists)
    /// Called by client when they detect that the NFT origin PDA already exists
    /// 
    /// This means the NFT was originally minted on Solana, transferred to another chain,
    /// and is now returning. The PDA contains:
    /// - original_mint: The first mint account ever created for this NFT on Solana
    /// - original_metadata: The first metadata account ever created for this NFT on Solana  
    /// - original_uri: The original metadata URI
    /// 
    /// We create NEW mint and metadata accounts (since originals were burned) but maintain
    /// the historical link to the original accounts as specified in GitHub issue #72
    pub fn restore_returning_nft(
        ctx: Context<RestoreReturningNft>,
        token_id: [u8; 32],
        updated_name: String,
        updated_symbol: String,
    ) -> Result<()> {
        require!(!ctx.accounts.config.is_paused, UniversalNftError::OperationNotAllowed);
        
        let clock = Clock::get()?;
        
        let nft_origin = &mut ctx.accounts.nft_origin;
        
        msg!("Restoring returning Solana NFT with original URI: {}", nft_origin.original_uri);
        
        // The original mint key and metadata are already stored in nft_origin:
        // - nft_origin.original_mint: The very first mint account created on Solana
        // - nft_origin.original_metadata: The very first metadata account created on Solana
        // - nft_origin.original_uri: The original metadata URI
        // 
        // We use these to maintain the link to the original token while creating
        // a new mint/metadata for the current restoration (since the original was burned)
        msg!(
            "Found original mint: {}, original metadata: {}", 
            nft_origin.original_mint, 
            nft_origin.original_metadata
        );
        
        // Create authority signer seeds for PDA-based operations
        let authority_bump = ctx.accounts.config.bump;
        let authority_seeds = &[
            UNIVERSAL_NFT_CONFIG_SEED,
            &[authority_bump],
        ];
        let authority_signer = &[&authority_seeds[..]];
        
        // 1. Create metadata account using original URI but updated name/symbol
        create_metadata_account(
            &ctx.accounts.metadata.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.config.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &updated_name,
            &updated_symbol,
            &nft_origin.original_uri,
        )?;
        
        // 2. Create master edition account
        create_master_edition_account(
            &ctx.accounts.master_edition.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.config.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.metadata.to_account_info(),
            &ctx.accounts.metadata_program.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
        )?;
        
        // 3. Mint NFT to recipient
        mint_nft_to_recipient(
            &ctx.accounts.mint,
            &ctx.accounts.token_account,
            &ctx.accounts.config.to_account_info(),
            &ctx.accounts.token_program,
            Some(authority_signer),
        )?;
        
        // 4. Update origin tracking - mark as returned to Solana
        nft_origin.mark_returned_to_solana(clock.unix_timestamp);
        
        // NOTE: We DON'T update original_mint and original_metadata here!
        // These should remain pointing to the very first mint/metadata on Solana
        // to maintain the historical link
        // The new mint/metadata are separate accounts for the current restoration.
        
        msg!(
            "NFT restored - Original mint: {}, New mint: {}, Original metadata: {}, New metadata: {}", 
            nft_origin.original_mint,
            ctx.accounts.mint.key(),
            nft_origin.original_metadata,
            ctx.accounts.metadata.key()
        );
        
        // Note: Anchor automatically serializes account changes, no manual serialization needed
        
        emit!(NftMinted {
            mint: ctx.accounts.mint.key(),
            token_id,
            owner: ctx.accounts.recipient.key(),
            uri: nft_origin.original_uri.clone(),
            timestamp: clock.unix_timestamp
        });
        
        emit!(OriginRestored {
            token_id,
            original_mint: nft_origin.original_mint, // This is the FIRST mint ever created on Solana
            new_mint: ctx.accounts.mint.key(),       // This is the current restoration mint
            owner: ctx.accounts.recipient.key(),
            timestamp: clock.unix_timestamp,
        });
        
        msg!("Returning Solana NFT successfully restored");
        Ok(())
    }

    /// Update program configuration (authority only)
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_authority: Option<Pubkey>,
        new_gateway_program: Option<Pubkey>,
        pause: Option<bool>,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let old_authority = config.authority;
        let old_gateway = config.gateway_program;
        
        if let Some(authority) = new_authority {
            config.authority = authority;
        }
        
        if let Some(gateway) = new_gateway_program {
            config.gateway_program = gateway;
        }
        
        if let Some(paused) = pause {
            config.is_paused = paused;
        }
        
        emit!(ProgramConfigUpdated {
            old_authority: Some(old_authority),
            new_authority,
            old_gateway_program: Some(old_gateway),
            new_gateway_program,
            updated_by: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Program configuration updated");
        Ok(())
    }

    /// Implement on_revert for handling failed cross-chain transactions
    /// Called by ZetaChain gateway when a cross-chain transaction fails
    /// This function restores the NFT state and compensates the user
    pub fn on_revert(
        ctx: Context<OnRevert>,
        amount: u64,    // Amount of SOL being reverted (if any)
        sender: Pubkey, // The account that triggered the original deposit/call from Solana
        data: Vec<u8>,  // Revert message data containing failure context
    ) -> Result<()> {
        require!(!ctx.accounts.config.is_paused, UniversalNftError::OperationNotAllowed);
        
        let clock = Clock::get()?;
        
        // Verify that the call is coming from ZetaChain gateway
        let current_ix = anchor_lang::solana_program::sysvar::instructions::get_instruction_relative(
            0,
            &ctx.accounts.instruction_sysvar_account.to_account_info(),
        ).map_err(|_| UniversalNftError::InvalidCaller)?;
        
        require!(
            current_ix.program_id == ctx.accounts.config.gateway_program,
            UniversalNftError::InvalidCaller
        );
        
        // Parse revert data to understand what failed
        let revert_reason = if data.is_empty() {
            "Unknown failure".to_string()
        } else {
            // Try to decode the revert message as UTF-8 string
            String::from_utf8(data.clone()).unwrap_or_else(|_| {
                format!("Binary revert data: {:?}", &data[..std::cmp::min(32, data.len())])
            })
        };
        
        // Check if we need to restore an NFT that was burned during the failed transfer
        let nft_origin = &mut ctx.accounts.nft_origin;
        if nft_origin.original_mint != Pubkey::default() && !nft_origin.is_on_solana {
            // This NFT was transferred off Solana but the transaction failed
            // We need to re-mint it back to the original owner
            
            // Create authority signer seeds for CPI calls
            let authority_bump = ctx.accounts.config.bump;
            let authority_seeds = &[
                UNIVERSAL_NFT_CONFIG_SEED,
                &[authority_bump],
            ];
            let authority_signer = &[&authority_seeds[..]];
            
            // Create metadata account for the restored NFT
            create_metadata_account(
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.config.to_account_info(), // Use config as authority
                &ctx.accounts.payer.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                &format!("Restored NFT"), // Name could be retrieved from origin if stored
                &format!("RNFT"),          // Symbol
                &nft_origin.original_uri,
            )?;
            
            // Create master edition account
            create_master_edition_account(
                &ctx.accounts.master_edition.to_account_info(),
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.config.to_account_info(), // Use config as authority
                &ctx.accounts.payer.to_account_info(),
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.metadata_program.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
            )?;
            
            // Mint NFT back to the original owner
            let mint_to_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(), // Use config as authority
                },
                authority_signer,
            );
            token::mint_to(mint_to_ctx, 1)?;
            
            // Update origin tracking - mark as restored to Solana
            nft_origin.mark_returned_to_solana(clock.unix_timestamp);
            
            msg!("NFT restored to original owner due to failed cross-chain transfer");
        }
        
        // If SOL was sent back, it's already handled by the gateway
        // The amount parameter tells us how much was refunded
        
        emit!(CrossChainTransferReverted {
            sender,
            amount,
            revert_reason: revert_reason.clone(),
            timestamp: clock.unix_timestamp,
            nft_restored: nft_origin.original_mint != Pubkey::default() && !nft_origin.is_on_solana,
        });
        
        msg!(
            "Cross-chain transaction reverted. Amount: {} lamports, Sender: {:?}, Reason: {}",
            amount,
            sender,
            revert_reason
        );
        
        Ok(())
    }
}