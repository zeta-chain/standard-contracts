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
// use std::io::Seek;

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
        config.next_token_id = 0;
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
    pub fn mint_nft(
        ctx: Context<MintNft>,
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
            None,
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
            None,
        )?;

        // Mint NFT to recipient
        mint_nft_to_recipient(
            &ctx.accounts.mint,
            &ctx.accounts.token_account,
            &ctx.accounts.authority,
            &ctx.accounts.token_program,
            None, // No authority signer needed for minting
        )?;
        
        // Generate token_id inside the program: hash(mint + slot + next_token_id)
        // This keeps it deterministic, unique, and avoids trusting client input.
        let slot = Clock::get()?.slot;
        let mut hasher = anchor_lang::solana_program::hash::Hasher::default();
        hasher.hash(ctx.accounts.mint.key().as_ref());
        hasher.hash(&slot.to_le_bytes());
        hasher.hash(&ctx.accounts.config.next_token_id.to_le_bytes());
        let token_hash = hasher.result();
        let token_id: [u8; 32] = token_hash.to_bytes();

        // Increment next_token_id after computing token_id
        ctx.accounts.config.next_token_id = ctx.accounts.config
            .next_token_id
            .checked_add(1)
            .ok_or(UniversalNftError::ArithmeticOverflow)?;

        // Create and initialize origin PDA [NFT_ORIGIN_SEED, token_id]
        let (expected_origin, origin_bump) = Pubkey::find_program_address(&[NFT_ORIGIN_SEED, &token_id], &crate::id());
        require_keys_eq!(expected_origin, ctx.accounts.nft_origin.key(), UniversalNftError::InvalidProgram);

        // Create account if needed using config PDA as payer
        if ctx.accounts.nft_origin.data_is_empty() || *ctx.accounts.nft_origin.owner != crate::id() {
            let rent = Rent::get()?;
            let space = NftOrigin::LEN as u64;
            let lamports = rent.minimum_balance(space as usize);
            let config_seeds: &[&[u8]] = &[UNIVERSAL_NFT_CONFIG_SEED, &[ctx.accounts.config.bump]];
            let origin_seeds: &[&[u8]] = &[NFT_ORIGIN_SEED, &token_id, &[origin_bump]];
            let signers: &[&[&[u8]]] = &[config_seeds, origin_seeds];
            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::CreateAccount { from: ctx.accounts.config.to_account_info(), to: ctx.accounts.nft_origin.to_account_info() },
                    signers,
                ),
                lamports,
                space,
                &crate::id(),
            )?;
        }

        // Write discriminator + NftOrigin data
        let mut data_ref = ctx.accounts.nft_origin.try_borrow_mut_data()?;
        let disc = <NftOrigin as anchor_lang::Discriminator>::DISCRIMINATOR;
        let mut tmp = Vec::with_capacity(8 + NftOrigin::LEN);
        tmp.extend_from_slice(&disc);
        let mut origin_data = NftOrigin::new(
            token_id,
            ctx.accounts.mint.key(),
            ctx.accounts.metadata.key(),
            uri.clone(),
            clock.unix_timestamp,
            origin_bump,
        );
        origin_data.try_serialize(&mut tmp).map_err(|_| UniversalNftError::InvalidDataFormat)?;
        data_ref[..tmp.len()].copy_from_slice(&tmp);

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
        require!(!ctx.accounts.pda.is_paused, UniversalNftError::OperationNotAllowed);

        // Verify call originates from the Gateway using instruction sysvar
        let ix_sysvar = anchor_lang::solana_program::sysvar::instructions::id();
        
        // The instruction sysvar must be passed in remaining accounts by the gateway CPI
        let ix_account = ctx
            .remaining_accounts
            .iter()
            .find(|ai| ai.key() == ix_sysvar)
            .ok_or(UniversalNftError::InvalidCaller)?;
        
        let current_ix = anchor_lang::solana_program::sysvar::instructions::get_instruction_relative(
            0,
            ix_account,
        ).map_err(|_| UniversalNftError::InvalidCaller)?;
        
        require!(
            current_ix.program_id == ctx.accounts.pda.gateway_program,
            UniversalNftError::InvalidCaller
        );

        // Optionally assert gateway PDA owner
        require!(
            *ctx.accounts.gateway_pda.owner == ctx.accounts.pda.gateway_program,
            UniversalNftError::InvalidCaller
        );

        // Decode message
        let (token_id, origin_chain, _recipient_bytes, metadata_uri, name, symbol) = decode_nft_data_with_recipient(&data)?;

        // Derive origin PDA expected address from token_id only
        let (expected_origin_key, origin_bump) = Pubkey::find_program_address(
            &[NFT_ORIGIN_SEED, &token_id],
            &crate::id(),
        );
        require_keys_eq!(
            expected_origin_key,
            ctx.accounts.nft_origin.key(),
            UniversalNftError::InvalidDataFormat
        );

        let clock = Clock::get()?;

        // Sanity checks for token accounts
        require_keys_eq!(ctx.accounts.pda_ata.owner, ctx.accounts.pda.key(), UniversalNftError::InvalidProgram);
        require_keys_eq!(ctx.accounts.pda_ata.mint, ctx.accounts.mint_account.key(), UniversalNftError::InvalidProgram);

        // Prepare signer seeds once
        let signer_seeds_raw = &[UNIVERSAL_NFT_CONFIG_SEED, &[ctx.accounts.pda.bump]];
        let signer_seeds: &[&[&[u8]]] = &[&signer_seeds_raw[..]];

        // 1) Ensure metadata + master edition exist first. This uses the program PDA as authority.
        // Doing this up front guarantees deterministic state and avoids partially-initialized origin records.
        if ctx.accounts.metadata.data_is_empty() {
            create_metadata_account(
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.mint_account.to_account_info(),
                &ctx.accounts.pda.to_account_info(),
                &ctx.accounts.pda.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                &name,
                &symbol,
                &metadata_uri,
                Some(signer_seeds),
            )?;
        }
        if ctx.accounts.master_edition.data_is_empty() {
            create_master_edition_account(
                &ctx.accounts.master_edition.to_account_info(),
                &ctx.accounts.mint_account.to_account_info(),
                &ctx.accounts.pda.to_account_info(),
                &ctx.accounts.pda.to_account_info(),
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.metadata_program.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                Some(signer_seeds),
            )?;
        }

        // 2) Load or create origin account backed by these accounts and the provided URI.
        let mut origin_data: NftOrigin;
        if ctx.accounts.nft_origin.to_account_info().owner == &crate::id() && !ctx.accounts.nft_origin.data_is_empty() {
            // Manually deserialize existing origin account to avoid lifetime issues
            let data_ref = ctx.accounts.nft_origin.try_borrow_data()?;
            require!(data_ref.len() >= 8, UniversalNftError::InvalidDataFormat);
            let mut bytes: &[u8] = &data_ref[8..]; // skip discriminator
            origin_data = NftOrigin::try_deserialize(&mut bytes)
                .map_err(|_| UniversalNftError::InvalidDataFormat)?;
        } else {
            origin_data = NftOrigin::new(
                token_id,
                ctx.accounts.mint_account.key(),
                ctx.accounts.metadata.key(),
                metadata_uri.clone(),
                clock.unix_timestamp,
                origin_bump,
            );

            let rent = Rent::get()?;
            let space = NftOrigin::LEN as u64;
            let lamports = rent.minimum_balance(space as usize);
            
            // Sign as both payer PDA and the new nft_origin PDA
            let config_seeds_raw: &[&[u8]] = &[UNIVERSAL_NFT_CONFIG_SEED, &[ctx.accounts.pda.bump]];
            let nft_origin_seeds: &[&[u8]] = &[
                NFT_ORIGIN_SEED,
                &token_id,
                &[origin_bump],
            ];
            let create_signers: &[&[&[u8]]] = &[&config_seeds_raw[..], nft_origin_seeds];

            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::CreateAccount {
                        from: ctx.accounts.pda.to_account_info(),
                        to: ctx.accounts.nft_origin.to_account_info(),
                    },
                    create_signers,
                ),
                lamports,
                space,
                &crate::id(),
            )?;
            
            let mut data_ref = ctx.accounts.nft_origin.try_borrow_mut_data()?;
            
            // Write discriminator + serialized data
            let disc = <NftOrigin as anchor_lang::Discriminator>::DISCRIMINATOR;
            let mut tmp = Vec::with_capacity(8 + NftOrigin::LEN);
            tmp.extend_from_slice(&disc);
            origin_data
                .try_serialize(&mut tmp)
                .map_err(|_| UniversalNftError::InvalidDataFormat)?;
            let end = tmp.len();
            data_ref[..end].copy_from_slice(&tmp);
        }

        // 3) Mint 1 to program_ata using config PDA as authority
        mint_nft_to_recipient(
            &ctx.accounts.mint_account,
            &ctx.accounts.pda_ata,
            &ctx.accounts.pda.to_account_info(),
            &ctx.accounts.token_program,
            Some(signer_seeds),
        )?;

        // Update origin flags (mark returned to Solana regardless of prior state)
        origin_data.mark_to_solana(clock.unix_timestamp);

        // 4) Re-serialize origin_data (in-place update)
        {
            let mut data_ref = ctx.accounts.nft_origin.try_borrow_mut_data()?;
            let mut tmp = Vec::with_capacity(NftOrigin::LEN);
            origin_data
                .try_serialize(&mut tmp)
                .map_err(|_| UniversalNftError::InvalidDataFormat)?;
            let pos = 8usize; // after discriminator
            let end = pos + tmp.len();
            let data_slice = &mut data_ref[pos..end];
            data_slice.copy_from_slice(&tmp);
        }

        emit!(NftMinted {
            mint: ctx.accounts.mint_account.key(),
            token_id,
            owner: ctx.accounts.pda.key(),
            uri: origin_data.original_uri.clone(),
            timestamp: clock.unix_timestamp
        });

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
                Some(authority_signer),
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
                Some(authority_signer),
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
            nft_origin.mark_to_solana(clock.unix_timestamp);

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