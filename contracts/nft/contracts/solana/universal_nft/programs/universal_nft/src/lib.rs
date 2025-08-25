use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{instructions::Instructions, SysvarId},
};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{burn, mint_to, Burn, Mint, MintTo, Token, TokenAccount},
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    instruction::{
        builders::{
            CreateMasterEditionV3Builder, CreateMetadataAccountV3Builder, SetAndVerifyCollectionBuilder,
        },
        InstructionBuilder,
    },
    state::{Collection, Creator, DataV2, Metadata},
    ID as MetadataID,
};

declare_id!("Un1v3rsa1Nft111111111111111111111111111111");

const GATEWAY_PROGRAM_ID: Pubkey = 
    solana_program::pubkey!("ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis");

mod state {
    use super::*;

    #[account]
    pub struct Config {
        pub admin: Pubkey,
        pub treasury: Pubkey,
        pub mint_auth: Pubkey,
        pub collection_mint: Pubkey,
        pub bumps: ConfigBumps,
        pub next_token_id: u64,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
    pub struct ConfigBumps {
        pub config: u8,
        pub treasury: u8,
        pub mint_auth: u8,
    }

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct NftOrigin {
        pub token_id: [u8; 32],
        pub original_solana_mint: Option<Pubkey>,
        pub mint_count: u64,
    }

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct ProcessedMessage {
        pub sender: [u8; 20],
        pub token_id: [u8; 32],
        pub nonce: u64,
    }
}

mod payload {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct Payload {
        pub token_id: [u8; 32],
        pub name: String,
        pub symbol: String,
        pub uri: String,
        pub recipient: Pubkey,
        pub origin_chain_id: u32,
        pub nonce: u64,
        pub original_solana_mint: Option<Pubkey>,
    }
}

mod utils {
    use super::*;

    pub fn check_gateway(instructions_sysvar: &AccountInfo) -> Result<()> {
        let ix_sysvar = Instructions::from_account_info(instructions_sysvar)?;
        let current_ix = Instructions::get_instruction_at(&ix_sysvar, 0)?;
        require!(
            current_ix.program_id == crate::ID,
            ErrorCode::InvalidGatewayProgram
        );

        let prev_ix = Instructions::get_instruction_at(&ix_sysvar, 1)?;
        require!(
            prev_ix.program_id == GATEWAY_PROGRAM_ID,
            ErrorCode::InvalidGatewayProgram
        );

        Ok(())
    }

    pub fn ensure_account_initialized(account: &AccountInfo) -> bool {
        account.data_len() > 0 && !account.data_is_empty()
    }

    pub fn create_pda_account<T: BorshSerialize>(
        payer: &AccountInfo,
        new_account: &AccountInfo,
        space: usize,
        seeds: &[&[u8]],
        bump: u8,
        owner: &Pubkey,
        system_program: &AccountInfo,
        data: T,
    ) -> Result<()> {
        let seeds_with_bump = [seeds, &[&[bump]]].concat();
        let seeds_slice = seeds_with_bump.iter().map(|s| s.as_slice()).collect::<Vec<_>>();

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_account.key,
                Rent::get()?.minimum_balance(space),
                space as u64,
                owner,
            ),
            &[payer.clone(), new_account.clone(), system_program.clone()],
            &[&seeds_slice[..]],
        )?;

        let mut data_ref = new_account.try_borrow_mut_data()?;
        data.serialize(&mut *data_ref)?;

        Ok(())
    }

    pub fn get_ata(wallet: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
        anchor_spl::associated_token::get_associated_token_address(wallet, mint)
    }

    pub fn create_ata(
        payer: &AccountInfo,
        wallet: &AccountInfo,
        mint: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
        associated_token_program: &AccountInfo,
        rent: &AccountInfo,
    ) -> Result<()> {
        invoke(
            &associated_token::create_associated_token_account(
                payer.key,
                wallet.key,
                mint.key,
                &anchor_spl::token::ID,
            ),
            &[
                payer.clone(),
                wallet.clone(),
                mint.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
                rent.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn create_metadata(
        metadata_account: &AccountInfo,
        mint: &AccountInfo,
        mint_authority: &AccountInfo,
        payer: &AccountInfo,
        update_authority: &AccountInfo,
        system_program: &AccountInfo,
        rent: &AccountInfo,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
        collection: Option<Collection>,
        uses: Option<mpl_token_metadata::state::Uses>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            creators,
            collection,
            uses,
        };

        let ix = CreateMetadataAccountV3Builder::new()
            .metadata(metadata_account.key())
            .mint(mint.key())
            .mint_authority(mint_authority.key())
            .payer(payer.key())
            .update_authority(update_authority.key(), update_authority_is_signer)
            .data(data_v2)
            .is_mutable(is_mutable)
            .build()
            .unwrap()
            .instruction();

        invoke_signed(
            &ix,
            &[
                metadata_account.clone(),
                mint.clone(),
                mint_authority.clone(),
                payer.clone(),
                update_authority.clone(),
                system_program.clone(),
                rent.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }

    pub fn create_master_edition(
        edition_account: &AccountInfo,
        mint: &AccountInfo,
        update_authority: &AccountInfo,
        mint_authority: &AccountInfo,
        metadata: &AccountInfo,
        payer: &AccountInfo,
        system_program: &AccountInfo,
        rent: &AccountInfo,
        token_program: &AccountInfo,
        max_supply: Option<u64>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let ix = CreateMasterEditionV3Builder::new()
            .edition(edition_account.key())
            .mint(mint.key())
            .update_authority(update_authority.key())
            .mint_authority(mint_authority.key())
            .metadata(metadata.key())
            .payer(payer.key())
            .max_supply(max_supply)
            .build()
            .unwrap()
            .instruction();

        invoke_signed(
            &ix,
            &[
                edition_account.clone(),
                mint.clone(),
                update_authority.clone(),
                mint_authority.clone(),
                metadata.clone(),
                payer.clone(),
                system_program.clone(),
                rent.clone(),
                token_program.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }

    pub fn set_and_verify_collection(
        metadata: &AccountInfo,
        collection_authority: &AccountInfo,
        payer: &AccountInfo,
        update_authority: &AccountInfo,
        collection_mint: &AccountInfo,
        collection_metadata: &AccountInfo,
        collection_master_edition: &AccountInfo,
        system_program: &AccountInfo,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let ix = SetAndVerifyCollectionBuilder::new()
            .metadata(metadata.key())
            .collection_authority(collection_authority.key())
            .payer(payer.key())
            .update_authority(update_authority.key())
            .collection_mint(collection_mint.key())
            .collection(collection_metadata.key())
            .collection_master_edition(collection_master_edition.key())
            .collection_authority_record(None)
            .build()
            .unwrap()
            .instruction();

        invoke_signed(
            &ix,
            &[
                metadata.clone(),
                collection_authority.clone(),
                payer.clone(),
                update_authority.clone(),
                collection_mint.clone(),
                collection_metadata.clone(),
                collection_master_edition.clone(),
                system_program.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid Gateway Program")]
    InvalidGatewayProgram,
    #[msg("Invalid Gateway Meta")]
    InvalidGatewayMeta,
    #[msg("Message Already Processed")]
    MessageAlreadyProcessed,
    #[msg("Invalid Token ID")]
    InvalidTokenId,
    #[msg("Invalid Mint")]
    InvalidMint,
    #[msg("Invalid Owner")]
    InvalidOwner,
    #[msg("Account Not Initialized")]
    AccountNotInitialized,
}

#[event]
pub struct OutboundPrepared {
    pub token_id: [u8; 32],
    pub mint: Pubkey,
}

#[event]
pub struct InboundReceived {
    pub token_id: [u8; 32],
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub sender: [u8; 20],
    pub origin_chain_id: u32,
}

#[program]
pub mod universal_nft {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        admin: Pubkey,
        collection_name: String,
        collection_symbol: String,
        collection_uri: String,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = admin;
        config.treasury = ctx.accounts.treasury.key();
        config.mint_auth = ctx.accounts.mint_auth.key();
        config.collection_mint = ctx.accounts.collection_mint.key();
        config.bumps = ctx.bumps.into();
        config.next_token_id = 1;

        // Create collection mint
        let mint_auth_seeds = &[b"mint_auth", &[ctx.bumps.mint_auth]];
        let mint_auth_signer = &[&mint_auth_seeds[..]];

        // Mint 1 token to treasury
        let cpi_accounts = MintTo {
            mint: ctx.accounts.collection_mint.to_account_info(),
            to: ctx.accounts.treasury_token_account.to_account_info(),
            authority: ctx.accounts.mint_auth.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, mint_auth_signer);
        mint_to(cpi_ctx, 1)?;

        // Create metadata
        let metadata_seeds = &[b"metadata", MetadataID.as_ref(), ctx.accounts.collection_mint.key().as_ref()];
        let (metadata_key, _) = Pubkey::find_program_address(metadata_seeds, &MetadataID);
        require!(metadata_key == ctx.accounts.collection_metadata.key(), ErrorCode::InvalidMint);

        utils::create_metadata(
            &ctx.accounts.collection_metadata,
            &ctx.accounts.collection_mint.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            collection_name,
            collection_symbol,
            collection_uri,
            None,
            0,
            false,
            true,
            None,
            None,
            mint_auth_signer,
        )?;

        // Create master edition
        let edition_seeds = &[
            b"metadata", 
            MetadataID.as_ref(), 
            ctx.accounts.collection_mint.key().as_ref(), 
            b"edition"
        ];
        let (edition_key, _) = Pubkey::find_program_address(edition_seeds, &MetadataID);
        require!(edition_key == ctx.accounts.collection_master_edition.key(), ErrorCode::InvalidMint);

        utils::create_master_edition(
            &ctx.accounts.collection_master_edition,
            &ctx.accounts.collection_mint.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.collection_metadata,
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            Some(0),
            mint_auth_signer,
        )?;

        Ok(())
    }

    pub fn mint_local(
        ctx: Context<MintLocal>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        
        // Generate token_id from mint pubkey, slot, and next_token_id
        let slot = Clock::get()?.slot;
        let mut token_id_data = Vec::new();
        token_id_data.extend_from_slice(&ctx.accounts.mint.key().to_bytes());
        token_id_data.extend_from_slice(&slot.to_le_bytes());
        token_id_data.extend_from_slice(&config.next_token_id.to_le_bytes());
        
        let token_id = anchor_lang::solana_program::hash::hash(&token_id_data).to_bytes();
        config.next_token_id += 1;

        // Create NFT origin record
        let nft_origin_seeds = &[b"nft_origin", &token_id];
        let (_, nft_origin_bump) = Pubkey::find_program_address(nft_origin_seeds, &crate::ID);
        
        let nft_origin_data = state::NftOrigin {
            token_id,
            original_solana_mint: Some(ctx.accounts.mint.key()),
            mint_count: 1,
        };
        
        utils::create_pda_account(
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.nft_origin,
            std::mem::size_of::<state::NftOrigin>(),
            nft_origin_seeds,
            nft_origin_bump,
            &crate::ID,
            &ctx.accounts.system_program.to_account_info(),
            nft_origin_data,
        )?;

        // Mint token
        let mint_auth_seeds = &[b"mint_auth", &[ctx.bumps.mint_auth]];
        let mint_auth_signer = &[&mint_auth_seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_auth.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, mint_auth_signer);
        mint_to(cpi_ctx, 1)?;

        // Create metadata
        let metadata_seeds = &[b"metadata", MetadataID.as_ref(), ctx.accounts.mint.key().as_ref()];
        let (metadata_key, _) = Pubkey::find_program_address(metadata_seeds, &MetadataID);
        require!(metadata_key == ctx.accounts.metadata.key(), ErrorCode::InvalidMint);

        let collection = Collection {
            verified: false,
            key: ctx.accounts.config.collection_mint,
        };

        utils::create_metadata(
            &ctx.accounts.metadata,
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            name,
            symbol,
            uri,
            None,
            0,
            false,
            true,
            Some(collection),
            None,
            mint_auth_signer,
        )?;

        // Create master edition
        let edition_seeds = &[
            b"metadata", 
            MetadataID.as_ref(), 
            ctx.accounts.mint.key().as_ref(), 
            b"edition"
        ];
        let (edition_key, _) = Pubkey::find_program_address(edition_seeds, &MetadataID);
        require!(edition_key == ctx.accounts.master_edition.key(), ErrorCode::InvalidMint);

        utils::create_master_edition(
            &ctx.accounts.master_edition,
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.metadata,
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            Some(0),
            mint_auth_signer,
        )?;

        // Set and verify collection
        utils::set_and_verify_collection(
            &ctx.accounts.metadata,
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.collection_mint.to_account_info(),
            &ctx.accounts.collection_metadata,
            &ctx.accounts.collection_master_edition,
            &ctx.accounts.system_program.to_account_info(),
            mint_auth_signer,
        )?;

        Ok(())
    }

    pub fn burn_and_prepare(ctx: Context<BurnAndPrepare>) -> Result<()> {
        // Read token_id from NftOrigin
        let nft_origin_data = ctx.accounts.nft_origin.try_borrow_data()?;
        let nft_origin = state::NftOrigin::try_from_slice(&nft_origin_data)?;
        
        // Verify the mint exists in NftOrigin
        require!(
            nft_origin.original_solana_mint.is_some() && 
            nft_origin.original_solana_mint.unwrap() == ctx.accounts.mint.key(),
            ErrorCode::InvalidMint
        );

        // Burn the token
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        burn(cpi_ctx, 1)?;

        // Emit event with token_id and mint
        emit!(OutboundPrepared {
            token_id: nft_origin.token_id,
            mint: ctx.accounts.mint.key(),
        });

        Ok(())
    }

    pub fn on_call(
        ctx: Context<OnCall>,
        sender: [u8; 20],
        amount: u64,
        data: Vec<u8>,
    ) -> Result<()> {
        // Verify the previous instruction was from the Gateway program
        utils::check_gateway(&ctx.accounts.instructions.to_account_info())?;

        // Verify gateway meta account
        let gateway_meta_seeds = &[b"meta"];
        let (expected_gateway_meta, _) = Pubkey::find_program_address(gateway_meta_seeds, &GATEWAY_PROGRAM_ID);
        require!(
            ctx.accounts.gateway_meta.key() == expected_gateway_meta,
            ErrorCode::InvalidGatewayMeta
        );

        // Decode payload
        let payload = payload::Payload::try_from_slice(&data)?;

        // Check if message already processed
        let processed_message_seeds = &[
            b"processed", 
            &sender, 
            &payload.token_id,
            &payload.nonce.to_le_bytes()
        ];
        let (processed_message_key, processed_message_bump) = 
            Pubkey::find_program_address(processed_message_seeds, &crate::ID);
        
        require!(
            processed_message_key == ctx.accounts.processed_message.key(),
            ErrorCode::InvalidMint
        );
        
        let is_processed_empty = ctx.accounts.processed_message.data_is_empty();
        
        if !is_processed_empty {
            // Read existing processed message and verify it's not a replay
            let processed_data = ctx.accounts.processed_message.try_borrow_data()?;
            let processed = state::ProcessedMessage::try_from_slice(&processed_data)?;
            
            require!(
                processed.sender != sender ||
                processed.token_id != payload.token_id ||
                processed.nonce != payload.nonce,
                ErrorCode::MessageAlreadyProcessed
            );
        }
        
        // Create processed message record
        let processed_message_data = state::ProcessedMessage {
            sender,
            token_id: payload.token_id,
            nonce: payload.nonce,
        };
        
        if is_processed_empty {
            utils::create_pda_account(
                &ctx.accounts.payer.to_account_info(),
                &ctx.accounts.processed_message,
                std::mem::size_of::<state::ProcessedMessage>(),
                processed_message_seeds,
                processed_message_bump,
                &crate::ID,
                &ctx.accounts.system_program.to_account_info(),
                processed_message_data,
            )?;
        }

        // Check if NFT origin exists
        let nft_origin_seeds = &[b"nft_origin", &payload.token_id];
        let (nft_origin_key, nft_origin_bump) = 
            Pubkey::find_program_address(nft_origin_seeds, &crate::ID);
        
        require!(
            nft_origin_key == ctx.accounts.nft_origin.key(),
            ErrorCode::InvalidMint
        );
        
        let is_nft_origin_empty = ctx.accounts.nft_origin.data_is_empty();
        
        let nft_origin_data = if is_nft_origin_empty {
            // Initialize new NFT origin
            state::NftOrigin {
                token_id: payload.token_id,
                original_solana_mint: payload.original_solana_mint,
                mint_count: 1,
            }
        } else {
            // Read and update existing NFT origin
            let nft_origin_existing = ctx.accounts.nft_origin.try_borrow_data()?;
            let mut nft_origin = state::NftOrigin::try_from_slice(&nft_origin_existing)?;
            
            require!(
                nft_origin.token_id == payload.token_id,
                ErrorCode::InvalidTokenId
            );
            
            nft_origin.mint_count += 1;
            nft_origin
        };
        
        if is_nft_origin_empty {
            utils::create_pda_account(
                &ctx.accounts.payer.to_account_info(),
                &ctx.accounts.nft_origin,
                std::mem::size_of::<state::NftOrigin>(),
                nft_origin_seeds,
                nft_origin_bump,
                &crate::ID,
                &ctx.accounts.system_program.to_account_info(),
                nft_origin_data,
            )?;
        } else {
            let mut nft_origin_ref = ctx.accounts.nft_origin.try_borrow_mut_data()?;
            nft_origin_data.serialize(&mut *nft_origin_ref)?;
        }

        // Create ATA if it doesn't exist
        let recipient_ata = utils::get_ata(&payload.recipient, &ctx.accounts.mint.key());
        
        if ctx.accounts.token_account.data_is_empty() {
            utils::create_ata(
                &ctx.accounts.payer.to_account_info(),
                &ctx.accounts.recipient.to_account_info(),
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.token_program.to_account_info(),
                &ctx.accounts.associated_token_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
            )?;
        }

        // Mint token
        let mint_auth_seeds = &[b"mint_auth", &[ctx.bumps.mint_auth]];
        let mint_auth_signer = &[&mint_auth_seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_auth.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, mint_auth_signer);
        mint_to(cpi_ctx, 1)?;

        // Create metadata
        let metadata_seeds = &[b"metadata", MetadataID.as_ref(), ctx.accounts.mint.key().as_ref()];
        let (metadata_key, _) = Pubkey::find_program_address(metadata_seeds, &MetadataID);
        require!(metadata_key == ctx.accounts.metadata.key(), ErrorCode::InvalidMint);

        let collection = Collection {
            verified: false,
            key: ctx.accounts.config.collection_mint,
        };

        utils::create_metadata(
            &ctx.accounts.metadata,
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            payload.name,
            payload.symbol,
            payload.uri,
            None,
            0,
            false,
            true,
            Some(collection),
            None,
            mint_auth_signer,
        )?;

        // Create master edition
        let edition_seeds = &[
            b"metadata", 
            MetadataID.as_ref(), 
            ctx.accounts.mint.key().as_ref(), 
            b"edition"
        ];
        let (edition_key, _) = Pubkey::find_program_address(edition_seeds, &MetadataID);
        require!(edition_key == ctx.accounts.master_edition.key(), ErrorCode::InvalidMint);

        utils::create_master_edition(
            &ctx.accounts.master_edition,
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.metadata,
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.rent.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            Some(0),
            mint_auth_signer,
        )?;

        // Set and verify collection
        utils::set_and_verify_collection(
            &ctx.accounts.metadata,
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.payer.to_account_info(),
            &ctx.accounts.mint_auth.to_account_info(),
            &ctx.accounts.collection_mint.to_account_info(),
            &ctx.accounts.collection_metadata,
            &ctx.accounts.collection_master_edition,
            &ctx.accounts.system_program.to_account_info(),
            mint_auth_signer,
        )?;

        // Emit event
        emit!(InboundReceived {
            token_id: payload.token_id,
            mint: ctx.accounts.mint.key(),
            recipient: payload.recipient,
            sender,
            origin_chain_id: payload.origin_chain_id,
        });

        Ok(())
    }

    pub fn on_revert(
        ctx: Context<OnRevert>,
        sender: [u8; 20],
        data: Vec<u8>,
    ) -> Result<()> {
        // Verify the previous instruction was from the Gateway program
        utils::check_gateway(&ctx.accounts.instructions.to_account_info())?;

        // Verify gateway meta account
        let gateway_meta_seeds = &[b"meta"];
        let (expected_gateway_meta, _) = Pubkey::find_program_address(gateway_meta_seeds, &GATEWAY_PROGRAM_ID);
        require!(
            ctx.accounts.gateway_meta.key() == expected_gateway_meta,
            ErrorCode::InvalidGatewayMeta
        );

        // Handle revert logic if needed
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(admin: Pubkey, collection_name: String, collection_symbol: String, collection_uri: String)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<state::Config>(),
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, state::Config>,

    #[account(
        init,
        payer = payer,
        space = 8,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = 8,
        seeds = [b"mint_auth"],
        bump
    )]
    pub mint_auth: SystemAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_auth,
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = collection_mint,
        associated_token::authority = treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Metadata account for collection
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    
    /// CHECK: Master edition account for collection
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct MintLocal<'info> {
    #[account(mut)]
    pub config: Account<'info, state::Config>,

    #[account(
        seeds = [b"mint_auth"],
        bump = config.bumps.mint_auth,
    )]
    pub mint_auth: SystemAccount<'info>,
    
    /// CHECK: NFT origin PDA, will be initialized in handler
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_auth,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Metadata account for NFT
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Master edition account for NFT
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    
    /// CHECK: Collection mint from config
    #[account(
        constraint = collection_mint.key() == config.collection_mint
    )]
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Collection metadata
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    
    /// CHECK: Collection master edition
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// CHECK: Recipient of the NFT
    pub recipient: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnAndPrepare<'info> {
    /// CHECK: NFT origin PDA
    pub nft_origin: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(sender: [u8; 20], amount: u64, data: Vec<u8>)]
pub struct OnCall<'info> {
    #[account(mut)]
    pub config: Account<'info, state::Config>,

    #[account(
        seeds = [b"mint_auth"],
        bump = config.bumps.mint_auth,
    )]
    pub mint_auth: SystemAccount<'info>,

    /// CHECK: NFT origin PDA, will be initialized or updated in handler
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,

    /// CHECK: Processed message PDA, will be initialized in handler
    #[account(mut)]
    pub processed_message: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_auth,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Token account, will be initialized if needed
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    
    /// CHECK: Metadata account for NFT
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Master edition account for NFT
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    
    /// CHECK: Collection mint from config
    #[account(
        constraint = collection_mint.key() == config.collection_mint
    )]
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Collection metadata
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    
    /// CHECK: Collection master edition
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// CHECK: Recipient account, used for ATA creation
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Gateway meta account verified in instruction
    #[account(owner = GATEWAY_PROGRAM_ID)]
    pub gateway_meta: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub instructions: Sysvar<'info, Instructions>,
}

#[derive(Accounts)]
#[instruction(sender: [u8; 20], data: Vec<u8>)]
pub struct OnRevert<'info> {
    #[account(mut)]
    pub config: Account<'info, state::Config>,

    /// CHECK: Gateway meta account verified in instruction
    #[account(owner = GATEWAY_PROGRAM_ID)]
    pub gateway_meta: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub instructions: Sysvar<'info, Instructions>,
}
