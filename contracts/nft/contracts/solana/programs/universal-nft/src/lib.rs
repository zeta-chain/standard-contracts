use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_spl::token::{self, Burn};

declare_id!("HrzackisVMLCjwJnu16KvkkHCsP8ridsZ6uQvfziyRsn");

// Module declarations
pub mod error;
pub mod util;
pub mod state;
pub mod event;
pub mod context;

use error::UniversalNftError;
use util::constants::*;
use util::{create_metadata_account, create_master_edition_account};
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
        // Critical validations for the gateway program configuration
        require!(gateway_program != Pubkey::default(), UniversalNftError::InvalidGatewayProgram);
        
        // Ensure the provided gateway account matches the argument and is executable
        require_keys_eq!(ctx.accounts.gateway_program.key(), gateway_program, UniversalNftError::InvalidGatewayProgram);
        require!(ctx.accounts.gateway_program.executable, UniversalNftError::InvalidGatewayProgram);

        // Validate provided gateway_pda is owned by this gateway program
        require!(
            *ctx.accounts.gateway_pda.owner == gateway_program,
            UniversalNftError::InvalidGatewayProgram
        );

        let config = &mut ctx.accounts.config;
        let clock = Clock::get()?;
        
        config.authority = ctx.accounts.authority.key();
        config.gateway_program = gateway_program;
        config.gateway_pda = ctx.accounts.gateway_pda.key();
        config.nonce = 0;
        config.next_token_id = 0;
        config.is_paused = false;
        config.created_at = clock.unix_timestamp;
        config.bump = ctx.bumps.config;
        
        emit!(ProgramConfigured {
            authority: config.authority,
            gateway_program: config.gateway_program,
            gateway_pda: config.gateway_pda,
            timestamp: config.created_at,
        });
        
        msg!("Universal NFT program initialized by: {:?}", config.authority);
        Ok(())
    }

    /// Reserve next token id for a specific mint and authority.
    /// Outputs a MintTicket PDA: [MINT_TICKET_SEED, mint, authority]
    pub fn reserve_next_token_id(ctx: Context<ReserveNextTokenId>) -> Result<()> {
        let slot = Clock::get()?.slot;
        let reserved_id = ctx.accounts.config.next_token_id;

        // Compute token_id = sha256(mint || slot_LE || reserved_id_LE)
        let mut hasher = anchor_lang::solana_program::hash::Hasher::default();
        hasher.hash(ctx.accounts.mint.key().as_ref());
        hasher.hash(&slot.to_le_bytes());
        hasher.hash(&reserved_id.to_le_bytes());
        let token_id = hasher.result().to_bytes();

        let ticket = &mut ctx.accounts.ticket;
        ticket.mint = ctx.accounts.mint.key();
        ticket.authority = ctx.accounts.authority.key();
        ticket.reserved_id = reserved_id;
        ticket.slot = slot;
        ticket.token_id = token_id;
        ticket.used = false;
        ticket.created_at = Clock::get()?.unix_timestamp;
        ticket.bump = ctx.bumps.ticket;

        // Increment global next_token_id
        ctx.accounts.config.next_token_id = ctx.accounts.config
            .next_token_id
            .checked_add(1)
            .ok_or(UniversalNftError::ArithmeticOverflow)?;

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

        // Use reservation ticket to determine token_id and enforce determinism
        require_keys_eq!(ctx.accounts.ticket.mint, ctx.accounts.mint.key(), UniversalNftError::InvalidProgram);
        require_keys_eq!(ctx.accounts.ticket.authority, ctx.accounts.authority.key(), UniversalNftError::InvalidProgram);
        require!(!ctx.accounts.ticket.used, UniversalNftError::OperationNotAllowed);

        // Recompute token_id from ticket
        let mut hasher = anchor_lang::solana_program::hash::Hasher::default();
        hasher.hash(ctx.accounts.mint.key().as_ref());
        hasher.hash(&ctx.accounts.ticket.slot.to_le_bytes());
        hasher.hash(&ctx.accounts.ticket.reserved_id.to_le_bytes());
        let token_id = hasher.result().to_bytes();
        require!(token_id == ctx.accounts.ticket.token_id, UniversalNftError::InvalidDataFormat);

        // Create metadata first (idempotent). Master edition MUST be after minting 1 supply.
        if ctx.accounts.metadata.data_is_empty() {
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
        }

        // Mint/token invariants
        require!(ctx.accounts.mint.decimals == 0, UniversalNftError::InvalidMint);
        require_keys_eq!(ctx.accounts.token_account.mint, ctx.accounts.mint.key(), UniversalNftError::InvalidMint);
        require_keys_eq!(ctx.accounts.token_account.owner, ctx.accounts.recipient.key(), UniversalNftError::InvalidRecipientAddress);

        // Ensure total supply is exactly 1 before creating master edition
        if ctx.accounts.mint.supply == 0 {
            // Mint NFT to recipient (will create ATA if helper does so)
            mint_nft_to_recipient(
                &ctx.accounts.mint,
                &ctx.accounts.token_account,
                &ctx.accounts.authority,
                &ctx.accounts.token_program,
                None,
            )?;
            // Reload accounts to see updated on-chain state after CPI
            ctx.accounts.mint.reload()?;
            ctx.accounts.token_account.reload()?;
        } else {
            // Defensive checks for idempotent retries
            require!(ctx.accounts.mint.supply == 1, UniversalNftError::InvalidTokenSupply);
            require!(ctx.accounts.token_account.amount == 1, UniversalNftError::InvalidTokenAmount);
        }

        // Now create master edition (requires supply == 1)
        if ctx.accounts.master_edition.data_is_empty() {
            // At this point, supply must be 1
            require!(ctx.accounts.mint.supply == 1, UniversalNftError::InvalidTokenSupply);
            create_master_edition_account(
                &ctx.accounts.master_edition.to_account_info(),
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.authority.to_account_info(),
                &ctx.accounts.payer.to_account_info(),
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.token_program.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                None,
            )?;
        }
        
        // Create and initialize origin PDA [NFT_ORIGIN_SEED, token_id]
        let (expected_origin, origin_bump) = Pubkey::find_program_address(&[NFT_ORIGIN_SEED, &token_id], &crate::id());
        require_keys_eq!(expected_origin, ctx.accounts.nft_origin.key(), UniversalNftError::InvalidProgram);

        // Create account if needed using payer as funder; sign as the new PDA only
        if ctx.accounts.nft_origin.data_is_empty() {
            let rent = Rent::get()?;
            let space = NftOrigin::SPACE as u64;
            let lamports = rent.minimum_balance(space as usize);
            let origin_seeds: &[&[u8]] = &[NFT_ORIGIN_SEED, &token_id, &[origin_bump]];
            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::CreateAccount { from: ctx.accounts.payer.to_account_info(), to: ctx.accounts.nft_origin.to_account_info() },
                    &[origin_seeds],
                ),
                lamports,
                space,
                &crate::id(),
            )?;

            // Write full account (discriminator + data) only on creation
            let mut data_ref = ctx.accounts.nft_origin.try_borrow_mut_data()?;
            let mut tmp = Vec::with_capacity(NftOrigin::SPACE);
            let origin_data = NftOrigin::new(
                token_id,
                ctx.accounts.mint.key(),
                ctx.accounts.metadata.key(),
                uri.clone(),
                clock.unix_timestamp,
                origin_bump,
            );

            // Serialize full account (includes discriminator)
            anchor_lang::prelude::AccountSerialize::try_serialize(&origin_data, &mut tmp)
                .map_err(|_| UniversalNftError::InvalidDataFormat)?;
            require!(tmp.len() <= NftOrigin::SPACE, UniversalNftError::InvalidDataFormat);
            data_ref.fill(0);
            data_ref[..tmp.len()].copy_from_slice(&tmp);

            emit!(OriginTracked {
                token_id,
                original_mint: ctx.accounts.mint.key(),
                original_metadata: ctx.accounts.metadata.key(),
                original_uri: uri.clone(),
                created_at: clock.unix_timestamp,
                timestamp: clock.unix_timestamp,
            });
        } else {
            // If already initialized and owned by this program, validate it matches expected values
            if *ctx.accounts.nft_origin.owner == crate::id() {
                let data_ref = ctx.accounts.nft_origin.try_borrow_data()?;
                require!(data_ref.len() >= 8, UniversalNftError::InvalidDataFormat);
                let mut bytes: &[u8] = &data_ref[8..]; // skip discriminator
                let existing = NftOrigin::try_deserialize(&mut bytes).map_err(|_| UniversalNftError::InvalidDataFormat)?;
                require!(existing.token_id == token_id, UniversalNftError::OriginConflict);
                require!(existing.original_mint == ctx.accounts.mint.key(), UniversalNftError::OriginConflict);
                require!(existing.original_metadata == ctx.accounts.metadata.key(), UniversalNftError::OriginConflict);
                require!(existing.original_uri == uri, UniversalNftError::OriginConflict); // TODO
                // Matches expected; proceed without rewriting (idempotent)
            } else {
                return err!(UniversalNftError::InvalidAccountOwner);
            }
        }

        emit!(NftMinted {
            mint: ctx.accounts.mint.key(),
            token_id,
            owner: ctx.accounts.recipient.key(),
            uri: uri.clone(),
            timestamp: clock.unix_timestamp
        });
        // Mark ticket as used only after successful mint + ME + origin
        ctx.accounts.ticket.used = true;
        
        // NOTE: Ticket will be auto-closed to authority via the close attribute in the context
        msg!("NFT minted successfully with token_id: {:?}", token_id);
        Ok(())
    }

    /// Transfer NFT intent to ZetaChain via gateway call (no SOL deposit)
    /// Burns the NFT on Solana and triggers the universal contract on ZetaChain
    pub fn transfer_to_zetachain(
        ctx: Context<TransferToZetachain>,
        token_id: [u8; 32],
        zetachain_universal_contract: [u8; 20], // ZetaChain universal NFT contract address
        final_destination_zrc20: [u8; 20], // Zero => stay on ZetaChain; non-zero => forward via this ZRC-20
        final_recipient: String, // Final recipient on destination chain
        sol_deposit_lamports: u64, // SOL to deposit for Zeta gas (lamports)
    ) -> Result<()> {
        require!(!ctx.accounts.config.is_paused, UniversalNftError::OperationNotAllowed);
        require!(!final_recipient.is_empty(), UniversalNftError::InvalidRecipientAddress);
        require!(final_recipient.len() <= MAX_RECIPIENT_LENGTH, UniversalNftError::InvalidRecipientAddress);
        // Require a reasonable deposit to cover 0.002 SOL fee + withdraw gas; caller selects value
        require!(sol_deposit_lamports >= 2_000_000, UniversalNftError::OperationNotAllowed);
        
        // Ensure the provided gateway PDA is actually owned by the configured gateway program
        require!(
            *ctx.accounts.gateway_pda.owner == ctx.accounts.config.gateway_program,
            UniversalNftError::InvalidGatewayProgram
        );
        
        // Defensive: ensure the provided token_id matches the stored one
        require!(ctx.accounts.nft_origin.token_id == token_id, UniversalNftError::InvalidDataFormat);
        
        // Validate nft_origin state and consistency
        require!(ctx.accounts.nft_origin.is_on_solana, UniversalNftError::NftNotOnSolana);
        require_keys_eq!(ctx.accounts.nft_origin.original_mint, ctx.accounts.mint.key(), UniversalNftError::InvalidMint);
        
        // Enforce strict ATA derivation for owner's token account
        let expected_owner_ata = anchor_spl::associated_token::get_associated_token_address(
            &ctx.accounts.owner.key(),
            &ctx.accounts.mint.key(),
        );
        require_keys_eq!(expected_owner_ata, ctx.accounts.token_account.key(), UniversalNftError::InvalidProgram);
        require_keys_eq!(ctx.accounts.token_account.owner, ctx.accounts.owner.key(), UniversalNftError::InvalidProgram);
        require_keys_eq!(ctx.accounts.token_account.mint, ctx.accounts.mint.key(), UniversalNftError::InvalidMint);
        require!(ctx.accounts.token_account.amount == 1, UniversalNftError::InvalidTokenAmount);

        let clock = Clock::get()?;
        let nft_origin = &mut ctx.accounts.nft_origin;
        
        // Build ABI-encoded message expected by ZEVM UniversalNFTCore.onCall FIRST
        // (address destination, address receiver, uint256 tokenId, string uri, address sender)
        // destination:
        //   - address(0) to stay/mint on ZetaChain
        //   - destination ZRC-20 address to forward to another connected EVM chain
        // receiver: parsed from final_recipient (ZEVM/EVM address)
        // sender: zero addr (no EVM address on Solana side)
        let receiver_addr = util::cross_chain_helpers::hex_string_to_address(&final_recipient)
            .map_err(|_| UniversalNftError::InvalidRecipientAddress)?;
        let uri = nft_origin.original_uri.clone();
        let sender_addr: [u8; 20] = [0u8; 20];
        let destination_zero: [u8; 20] = [0u8; 20];
        // Decide stay/forward purely from final_destination_zrc20
        let is_stay_on_zetachain = final_destination_zrc20 == destination_zero;
        let cross_chain_message = util::gateway_helpers::encode_evm_nft_message(
            if is_stay_on_zetachain {
                destination_zero
            } else {
                final_destination_zrc20
            },
            receiver_addr,
            token_id,
            &uri,
            sender_addr,
        );

        require!(ctx.accounts.mint.decimals == 0, UniversalNftError::InvalidMint);
        require!(ctx.accounts.mint.supply == 1, UniversalNftError::InvalidTokenSupply);
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
        nft_origin.mark_transferred_off_solana(clock.unix_timestamp);
        
        // Call ZetaChain gateway `deposit_and_call` via CPI, depositing SOL for gas and invoking ZEVM
        // This makes a cross-program invocation to the ZetaChain gateway program,
        // which handles calling the universal contract on ZetaChain.
        let gateway_pda_info = ctx.accounts.gateway_pda.to_account_info();
        let owner_info = ctx.accounts.owner.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();
        
        // Encode the receiver (ZetaChain universal contract) as fixed 20 bytes
        let receiver_bytes: [u8; 20] = zetachain_universal_contract;

        // Create CPI instruction to ZetaChain gateway: deposit_and_call(amount, receiver, message, revert_options=None)
        let call_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.config.gateway_program, // validated in accounts
            accounts: vec![
                AccountMeta::new(owner_info.key(), true),        // payer/signer supplying lamports
                AccountMeta::new(gateway_pda_info.key(), false), // Gateway PDA writable per docs
                AccountMeta::new_readonly(system_program_info.key(), false), // System Program
            ],
            data: util::gateway_helpers::encode_gateway_deposit_and_call_ix_data(sol_deposit_lamports, receiver_bytes, &cross_chain_message),
        };
        
        // TODO: Remove: Diagnostics: Print everything needed for the CPI invoke ---
        // Hex helpers
        let to_hex = |bytes: &[u8]| -> String {
            let mut s = String::with_capacity(bytes.len() * 2);
            for b in bytes {
                use core::fmt::Write as _;
                let _ = write!(&mut s, "{:02x}", b);
            }
            s
        };
        let receiver_hex = to_hex(&receiver_bytes);
        let token_id_hex = to_hex(&token_id);
        let msg_hex = to_hex(&cross_chain_message);

        msg!(
            "CPI invoke -> program_id (gateway): {}",
            ctx.accounts.config.gateway_program
        );
        msg!(
            "  Accounts[0] owner (payer/signer): {}, is_signer=true, is_writable=true",
            owner_info.key()
        );
        msg!(
            "  Accounts[1] gateway_pda: {}, is_signer=false, is_writable=true",
            gateway_pda_info.key()
        );
        msg!(
            "  Accounts[2] system_program: {}, is_signer=false, is_writable=false",
            system_program_info.key()
        );
        msg!("  Method: deposit_and_call");
        msg!("  Args.sol_deposit_lamports: {}", sol_deposit_lamports);
        msg!("  Args.receiver (Zeta universal, 20 bytes): 0x{}", receiver_hex);
        let dest_hex = if is_stay_on_zetachain { String::from("0x0000000000000000000000000000000000000000") } else { to_hex(&final_destination_zrc20) };
        msg!(
            "  Args.message fields (ZEVM onCall): destination={}, receiver=0x{}, tokenId=0x{}, uri='{}', sender=0x{}",
            dest_hex,
            to_hex(&receiver_addr),
            token_id_hex,
            uri,
            to_hex(&sender_addr)
        );
        msg!(
            "  Encoded message length: {} bytes (0x{})",
            cross_chain_message.len(),
            msg_hex
        );
        msg!(
            "  High-level params: is_stay_on_zetachain={}, final_recipient='{}'",
            is_stay_on_zetachain,
            final_recipient
        );

        // Execute CPI call to ZetaChain gateway, transferring lamports from owner to gateway as deposit
        anchor_lang::solana_program::program::invoke(
            &call_ix,
            &[
                owner_info,
                gateway_pda_info,
                system_program_info,
            ],
        )?;
        
        let config = &mut ctx.accounts.config;
        config.nonce = config.nonce.checked_add(1).ok_or(UniversalNftError::ArithmeticOverflow)?;
        
        emit!(CrossChainTransferInitiated {
            token_id,
            source_chain: SOLANA_CHAIN_ID,
            destination_chain: 0, // 0 => stay on ZetaChain or unknown; telemetry only
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
            -1,
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

        // Decode message from ZEVM universal contract: (address receiver, uint256 tokenId, string uri, uint256 amount, address sender)
        let (token_id, _receiver_evm, metadata_uri, _sender_evm) = util::cross_chain_helpers::decode_evm_abi_nft_message(&data)?;

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
        
        // Enforce ATA derivation without requiring associated_token_program account in context
        let expected_ata = anchor_spl::associated_token::get_associated_token_address(
            &ctx.accounts.pda.key(),
            &ctx.accounts.mint_account.key(),
        );

        require_keys_eq!(
            expected_ata,
            ctx.accounts.pda_ata.key(),
            UniversalNftError::InvalidProgram
        );

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
                &ctx.accounts.token_program.to_account_info(),
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
            let space = NftOrigin::SPACE as u64;
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
            
            // Write full account (discriminator + data)
            let mut tmp = Vec::with_capacity(NftOrigin::SPACE);
            anchor_lang::prelude::AccountSerialize::try_serialize(&origin_data, &mut tmp)
                .map_err(|_| UniversalNftError::InvalidDataFormat)?;
            require!(tmp.len() <= NftOrigin::SPACE, UniversalNftError::InvalidDataFormat);
            data_ref.fill(0);
            data_ref[..tmp.len()].copy_from_slice(&tmp);
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

        // 4) Re-serialize origin_data (full account update)
        {
            let mut data_ref = ctx.accounts.nft_origin.try_borrow_mut_data()?;
            let mut tmp = Vec::with_capacity(NftOrigin::SPACE);
            anchor_lang::prelude::AccountSerialize::try_serialize(&origin_data, &mut tmp)
                .map_err(|_| UniversalNftError::InvalidDataFormat)?;
            require!(tmp.len() <= NftOrigin::SPACE, UniversalNftError::InvalidDataFormat);
            data_ref.fill(0);
            data_ref[..tmp.len()].copy_from_slice(&tmp);
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
        new_gateway_pda: Option<Pubkey>,
        pause: Option<bool>,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let old_authority = config.authority;
        let old_gateway = config.gateway_program;
        let old_gateway_pda = config.gateway_pda;
        
        // Validate and update authority if requested
        if let Some(authority) = new_authority {
            require!(authority != Pubkey::default(), UniversalNftError::InvalidAuthority);
            if authority != config.authority {
                config.authority = authority;
            }
        }
        
        // Validate and update gateway program if requested
        if let Some(gateway) = new_gateway_program {
            // Disallow zero pubkey; optionally check executability if the account is provided
            require!(gateway != Pubkey::default(), UniversalNftError::InvalidGatewayProgram);
            if let Some(acc) = ctx.remaining_accounts.iter().find(|a| a.key() == gateway) {
                // If the caller supplied the gateway account, ensure it's a program (executable)
                require!(acc.executable, UniversalNftError::InvalidGatewayProgram);
            }
            if gateway != config.gateway_program {
                config.gateway_program = gateway;
            }
        }

        // Update gateway_pda if provided; must be owned by current (possibly updated) gateway program
        if let Some(gpda) = new_gateway_pda {
            require!(gpda != Pubkey::default(), UniversalNftError::InvalidGatewayProgram);
            if let Some(acc) = ctx.remaining_accounts.iter().find(|a| a.key() == gpda) {
                require!(acc.owner == &config.gateway_program, UniversalNftError::InvalidGatewayProgram);
            }
            if gpda != config.gateway_pda {
                config.gateway_pda = gpda;
            }
        }
        
        if let Some(paused) = pause {
            config.is_paused = paused;
        }
        
        emit!(ProgramConfigUpdated {
            old_authority: Some(old_authority),
            new_authority,
            old_gateway_program: Some(old_gateway),
            new_gateway_program,
            old_gateway_pda: Some(old_gateway_pda),
            new_gateway_pda,
            updated_by: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Program configuration updated");
        Ok(())
    }
}