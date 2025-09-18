use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::UniversalNftConfig;
use crate::util::bridge_constants::METAPLEX_TOKEN_METADATA_PROGRAM_ID;

#[derive(Accounts)]
pub struct CrossChainCallback<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECKED: This is a asset tracker account
    #[account(mut)]
    pub asset_tracker: UncheckedAccount<'info>,
    /// CHECKED: This is a program token account
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"config"],
        bump = config.pda_bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    /// CHECKED: This is a metadata account
    #[account(
        mut,
        seeds = [
            b"metadata",
            METAPLEX_TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = METAPLEX_TOKEN_METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECKED: This is a master edition account
    #[account(
        mut,
        seeds = [
            b"metadata",
            METAPLEX_TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
            b"edition",
        ],
        bump,
        seeds::program = METAPLEX_TOKEN_METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECKED: This is a gateway verifier account
    #[account(
        constraint = gateway_verifier.key() == config.zeta_gateway_verifier,
        constraint = *gateway_verifier.owner == config.zeta_gateway_program_id
    )]
    pub gateway_verifier: UncheckedAccount<'info>,

    /// CHECKED: This is a metadata program account
    #[account(constraint = metadata_program.key() == METAPLEX_TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CrossChainCallback<'info> {
    pub fn handle_cross_chain_callback(
        ctx: Context<Self>,
        _sol_amount: u64, 
        _zetachain_contract: [u8; 20], 
        encoded_data: Vec<u8>, 
    ) -> Result<()> {
        require!(!ctx.accounts.config.paused, crate::errors::Errors::OperationNotAllowed);

        Self::verify_gateway_origin(&ctx)?;

        let (nft_id, origin_chain, _recipient_bytes, metadata_uri, name, symbol) = 
            Self::decode_and_validate_message(&encoded_data)?;

        Self::validate_accounts(&ctx, nft_id)?;

        let clock = Clock::get()?;

        Self::ensure_metadata_accounts(&ctx, &name, &symbol, &metadata_uri)?;

        let mut origin_data = Self::load_or_create_origin_account(&ctx, nft_id, &metadata_uri, clock.unix_timestamp)?;

        Self::mint_nft_token(&ctx)?;

        Self::update_origin_tracking(&ctx, &mut origin_data)?;

        Self::emit_recovery_event(&ctx, nft_id, &metadata_uri, clock.unix_timestamp, origin_chain)?;

        Ok(())
    }

    fn verify_gateway_origin(ctx: &Context<Self>) -> Result<()> {
        crate::callbacks::callback_validator::CallbackValidator::verify_gateway_origin(
            &ctx.remaining_accounts,
            ctx.accounts.config.zeta_gateway_program_id,
        )
    }

    fn decode_and_validate_message(encoded_data: &[u8]) -> Result<([u8; 32], u64, [u8; 20], String, String, String)> {
        let (nft_id, origin_chain, recipient_bytes, metadata_uri, name, symbol) = 
            crate::util::CrossChainDataDecoder::decode_nft_data(encoded_data)?;

        crate::util::CrossChainDataDecoder::validate_decoded_data(&name, &symbol, &metadata_uri)?;

        Ok((nft_id, origin_chain, recipient_bytes, metadata_uri, name, symbol))
    }

    fn validate_accounts(ctx: &Context<Self>, nft_id: [u8; 32]) -> Result<()> {
        crate::callbacks::callback_validator::CallbackValidator::validate_asset_tracker_pda(
            &ctx.accounts.asset_tracker.to_account_info(),
            nft_id,
            crate::id(),
        )?;

        crate::callbacks::callback_validator::CallbackValidator::validate_token_account(
            &ctx.accounts.program_token_account.to_account_info(),
            ctx.accounts.config.key(),
            ctx.accounts.mint.key(),
        )
    }

    fn ensure_metadata_accounts(
        ctx: &Context<Self>,
        name: &str,
        symbol: &str,
        metadata_uri: &str,
    ) -> Result<()> {
        let signer_seeds_raw = &[crate::util::bridge_constants::UNIVERSAL_NFT_CONFIG_SEED, &[ctx.accounts.config.pda_bump]];
        let signer_seeds: &[&[&[u8]]] = &[&signer_seeds_raw[..]];

        if ctx.accounts.metadata.data_is_empty() {
            crate::operations::metadata_operations::initialize_metadata_account(
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.config.to_account_info(),
                &ctx.accounts.config.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                name,
                symbol,
                metadata_uri,
                Some(signer_seeds),
            )?;
        }

        if ctx.accounts.master_edition.data_is_empty() {
            crate::operations::metadata_operations::initialize_master_edition_account(
                &ctx.accounts.master_edition.to_account_info(),
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.config.to_account_info(),
                &ctx.accounts.config.to_account_info(),
                &ctx.accounts.metadata.to_account_info(),
                &ctx.accounts.token_program.to_account_info(),
                &ctx.accounts.system_program.to_account_info(),
                &ctx.accounts.rent.to_account_info(),
                Some(signer_seeds),
            )?;
        }

        Ok(())
    }

    fn load_or_create_origin_account(
        ctx: &Context<Self>,
        nft_id: [u8; 32],
        metadata_uri: &str,
        timestamp: i64,
    ) -> Result<crate::state::UniversalNftOrigin> {
        let (_, tracker_bump) = Pubkey::find_program_address(
            &[b"asset_tracker", nft_id.as_ref()],
            &crate::id(),
        );

        if ctx.accounts.asset_tracker.owner == &crate::id() && !ctx.accounts.asset_tracker.data_is_empty() {
            Self::load_existing_origin_account(&ctx.accounts.asset_tracker)
        } else {
            Self::create_new_origin_account(ctx, nft_id, metadata_uri, timestamp, tracker_bump)
        }
    }

    fn load_existing_origin_account(asset_tracker: &AccountInfo) -> Result<crate::state::UniversalNftOrigin> {
        let data_ref = asset_tracker.try_borrow_data()?;
        require!(data_ref.len() >= 8, crate::errors::Errors::InvalidDataFormat);
        let mut bytes: &[u8] = &data_ref[8..]; 
        crate::state::UniversalNftOrigin::try_deserialize(&mut bytes)
            .map_err(|_| crate::errors::Errors::InvalidDataFormat.into())
    }

    fn create_new_origin_account(
        ctx: &Context<Self>,
        nft_id: [u8; 32],
        metadata_uri: &str,
        timestamp: i64,
        tracker_bump: u8,
    ) -> Result<crate::state::UniversalNftOrigin> {
        let origin_data = crate::state::UniversalNftOrigin {
            nft_id,
            original_mint: ctx.accounts.mint.key(),
            original_metadata: ctx.accounts.metadata.key(),
            original_uri: metadata_uri.to_string(),
            is_on_solana: true,
            created_at: timestamp,
            transferred_at: None,
            bump_seed: tracker_bump,
        };

        let rent = Rent::get()?;
        let space = crate::state::UniversalNftOrigin::INIT_SPACE as u64;
        let lamports = rent.minimum_balance(space as usize);
        
        let config_seeds_raw: &[&[u8]] = &[crate::util::bridge_constants::UNIVERSAL_NFT_CONFIG_SEED, &[ctx.accounts.config.pda_bump]];
        let asset_tracker_seeds: &[&[u8]] = &[
            b"asset_tracker",
            &nft_id,
            &[tracker_bump],
        ];
        let create_signers: &[&[&[u8]]] = &[&config_seeds_raw[..], asset_tracker_seeds];

        anchor_lang::system_program::create_account(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::CreateAccount {
                    from: ctx.accounts.config.to_account_info(),
                    to: ctx.accounts.asset_tracker.to_account_info(),
                },
                create_signers,
            ),
            lamports,
            space,
            &crate::id(),
        )?;

        Self::write_origin_account_data(&ctx.accounts.asset_tracker, &origin_data)?;

        Ok(origin_data)
    }

    fn write_origin_account_data(
        asset_tracker: &AccountInfo,
        origin_data: &crate::state::UniversalNftOrigin,
    ) -> Result<()> {
        let mut data_ref = asset_tracker.try_borrow_mut_data()?;
        let mut tmp = Vec::with_capacity(crate::state::UniversalNftOrigin::INIT_SPACE);
        anchor_lang::prelude::AccountSerialize::try_serialize(origin_data, &mut tmp)
            .map_err(|_| crate::errors::Errors::InvalidDataFormat)?;
        require!(tmp.len() <= crate::state::UniversalNftOrigin::INIT_SPACE, crate::errors::Errors::InvalidDataFormat);
        data_ref.fill(0);
        data_ref[..tmp.len()].copy_from_slice(&tmp);
        Ok(())
    }

    fn mint_nft_token(ctx: &Context<Self>) -> Result<()> {
        let signer_seeds_raw = &[crate::util::bridge_constants::UNIVERSAL_NFT_CONFIG_SEED, &[ctx.accounts.config.pda_bump]];
        let signer_seeds: &[&[&[u8]]] = &[&signer_seeds_raw[..]];

        crate::operations::token_operations::generate_token_unit_for_recipient(
            &ctx.accounts.mint,
            &ctx.accounts.program_token_account,
            &ctx.accounts.config.to_account_info(),
            &ctx.accounts.token_program,
            Some(signer_seeds),
        )
    }

    fn update_origin_tracking(
        ctx: &Context<Self>,
        origin_data: &mut crate::state::UniversalNftOrigin,
    ) -> Result<()> {
        origin_data.is_on_solana = true;
        origin_data.transferred_at = None;

        Self::write_origin_account_data(&ctx.accounts.asset_tracker, origin_data)
    }

    fn emit_recovery_event(
        ctx: &Context<Self>,
        nft_id: [u8; 32],
        metadata_uri: &str,
        timestamp: i64,
        origin_chain: u64,
    ) -> Result<()> {
        msg!(
            "NFT recovered from cross-chain bridge\nMint: {}\nNFT ID: {:?}\nOwner: {}\nURI: {}\nTimestamp: {}\nOrigin chain: {}",
            ctx.accounts.mint.key(),
            nft_id,
            ctx.accounts.config.key(),
            metadata_uri,
            timestamp,
            origin_chain
        );  

        Ok(())
    }
}

