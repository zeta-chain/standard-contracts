use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::{
    state::{UniversalNftConfig, TokenReservation, UniversalNftOrigin},
    errors::Errors,
    operations::{generate_token_unit_for_recipient, initialize_metadata_account, initialize_master_edition_account},
};

const NFT_URI_LENGTH_LIMIT: usize = 200;
const NFT_NAME_LENGTH_LIMIT: usize = 32;
const NFT_SYMBOL_LENGTH_LIMIT: usize = 10;
const METAPLEX_TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

#[derive(Accounts)]
pub struct MintUniversalNft<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.pda_bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    /// Universal NFT voucher created via allocate_token_id; consumed here
    #[account(
        mut,
        seeds = [b"token_reservation", mint.key().as_ref(), authority.key().as_ref()],
        bump = reservation.bump_seed,
        constraint = !reservation.is_consumed @ Errors::VoucherAlreadyUsed,
        close = authority,
    )]
    pub reservation: Account<'info, TokenReservation>,

    /// Origin PDA passed but initialized in the handler using program-generated asset_id.
    /// The handler recomputes asset_id from voucher.{block_slot,reserved_id} and requires PDA match.
    /// CHECK: Verified in handler
    #[account(mut)]
    pub asset_origin: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    /// Metadata account derived from mint - validated for security
    /// CHECK: This account is derived from the mint using seeds, ensuring it's the correct metadata account
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
    
    /// Master edition account derived from mint - validated for security
    /// CHECK: This account is derived from the mint using seeds, ensuring it's the correct master edition account
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
    
    /// Deterministic recipient ATA; program initializes it if missing
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Validated recipient account - must not be default pubkey
    #[account(
        constraint = recipient.key() != Pubkey::default() @ Errors::InvalidRecipientAddress
    )]
    pub recipient: SystemAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Validated Metaplex Token Metadata program
    /// CHECK: This is verified to be the correct Metaplex Token Metadata program ID
    #[account(
        constraint = metadata_program.key() == METAPLEX_TOKEN_METADATA_PROGRAM_ID @ Errors::InvalidProgram
    )]
    pub metadata_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> MintUniversalNft<'info> {
    pub fn mint_universal_nft(
        mut ctx: Context<Self>,
        name: String,
        symbol: String,
        uri: String
    ) -> Result<()> {
        // Validate inputs
        Self::validate_inputs(&name, &symbol, &uri, &ctx.accounts.config)?;
        
        // Calculate required rent for account creation
        Self::validate_rent_requirements(&ctx.accounts)?;
        
        let clock = Clock::get()?;

        // Use voucher to determine nft_id and enforce determinism
        Self::validate_voucher(&ctx.accounts.reservation, &ctx.accounts.mint, &ctx.accounts.authority)?;

        // Compute NFT ID from voucher
        let nft_id = Self::compute_nft_id(&ctx.accounts.mint, &ctx.accounts.reservation)?;

        // Create metadata if needed
        Self::create_metadata_if_needed(&ctx.accounts, &name, &symbol, &uri)?;

        // Validate mint and token accounts
        Self::validate_mint_and_token(&ctx.accounts.mint, &ctx.accounts.token_account, &ctx.accounts.recipient)?;

        // Mint NFT token if needed
        Self::mint_nft_token_if_needed(&mut ctx.accounts)?;

        // Create master edition if needed
        Self::create_master_edition_if_needed(&ctx.accounts)?;
        
        // Create and initialize origin PDA
        Self::create_or_validate_origin_pda(&ctx.accounts, &nft_id, &uri, clock.unix_timestamp)?;

        // Mark voucher as consumed
        ctx.accounts.reservation.is_consumed = true;
        
        msg!("Universal NFT minted successfully with nft_id: {:?}", nft_id);
        Ok(())
    }

    /// Validate input parameters for Universal NFT minting
    fn validate_inputs(
        name: &str,
        symbol: &str,
        uri: &str,
        config: &UniversalNftConfig,
    ) -> Result<()> {
        require!(name.len() <= NFT_NAME_LENGTH_LIMIT, Errors::NameTooLong);
        require!(symbol.len() <= NFT_SYMBOL_LENGTH_LIMIT, Errors::SymbolTooLong);
        require!(uri.len() <= NFT_URI_LENGTH_LIMIT, Errors::UriTooLong);
        require!(!config.paused, Errors::ProgramPaused);
        Ok(())
    }

    /// Validate voucher for Universal NFT minting
    fn validate_voucher(
        voucher: &TokenReservation,
        mint: &Account<'_, Mint>,
        authority: &AccountInfo,
    ) -> Result<()> {
        require_keys_eq!(voucher.mint_address, mint.key(), Errors::InvalidProgram);
        require_keys_eq!(voucher.creator, authority.key(), Errors::InvalidProgram);
        require!(!voucher.is_consumed, Errors::VoucherAlreadyUsed);
        Ok(())
    }

    /// Validate mint and token account for Universal NFT minting
    fn validate_mint_and_token(
        mint: &Account<'_, Mint>,
        token_account: &Account<'_, TokenAccount>,
        recipient: &AccountInfo,
    ) -> Result<()> {
        require!(mint.decimals == 0, Errors::InvalidMint);
        require_keys_eq!(token_account.mint, mint.key(), Errors::InvalidMint);
        require_keys_eq!(token_account.owner, recipient.key(), Errors::InvalidRecipientAddress);
        Ok(())
    }

    /// Validate rent requirements for account creation
    fn validate_rent_requirements(accounts: &MintUniversalNft<'info>) -> Result<()> {
        let rent_sys = Rent::get()?;
        let mut required_lamports: u64 = 0;
        
        if accounts.metadata.data_is_empty() {
            required_lamports = required_lamports
                .saturating_add(rent_sys.minimum_balance(1024));
        }
        if accounts.master_edition.data_is_empty() {
            required_lamports = required_lamports
                .saturating_add(rent_sys.minimum_balance(512));
        }
        if accounts.asset_origin.data_is_empty() {
            required_lamports = required_lamports
                .saturating_add(rent_sys.minimum_balance(UniversalNftOrigin::INIT_SPACE));
        }
        
        if required_lamports > 0 {
            require!(accounts.payer.lamports() >= required_lamports, Errors::InsufficientRent);
        }
        
        Ok(())
    }

    /// Compute NFT ID from reservation data
    fn compute_nft_id(
        mint: &Account<'_, Mint>,
        reservation: &TokenReservation,
    ) -> Result<[u8; 32]> {
        let mut hasher = anchor_lang::solana_program::hash::Hasher::default();
        hasher.hash(mint.key().as_ref());
        hasher.hash(&reservation.block_slot.to_le_bytes());
        hasher.hash(&reservation.reserved_id.to_le_bytes());
        let nft_id = hasher.result().to_bytes();
        require!(nft_id == reservation.token_hash, Errors::InvalidDataFormat);
        Ok(nft_id)
    }

    /// Create metadata account if it doesn't exist
    fn create_metadata_if_needed(
        accounts: &MintUniversalNft<'info>,
        name: &str,
        symbol: &str,
        uri: &str,
    ) -> Result<()> {
        if accounts.metadata.data_is_empty() {
            initialize_metadata_account(
                &accounts.metadata.to_account_info(),
                &accounts.mint.to_account_info(),
                &accounts.authority.to_account_info(),
                &accounts.payer.to_account_info(),
                &accounts.system_program.to_account_info(),
                &accounts.rent.to_account_info(),
                name,
                symbol,
                uri,
                None,
            )?;
        }
        Ok(())
    }

    /// Mint NFT token if supply is 0
    fn mint_nft_token_if_needed(accounts: &mut MintUniversalNft<'info>) -> Result<()> {
        if accounts.mint.supply == 0 {
            // Generate token unit for recipient (will create ATA if helper does so)
            generate_token_unit_for_recipient(
                &accounts.mint,
                &accounts.token_account,
                &accounts.authority,
                &accounts.token_program,
                None,
            )?;
            // Reload accounts to see updated on-chain state after CPI
            accounts.mint.reload()?;
            accounts.token_account.reload()?;
        } else {
            // Defensive checks for idempotent retries
            require!(accounts.mint.supply == 1, Errors::InvalidTokenSupply);
            require!(accounts.token_account.amount == 1, Errors::InvalidTokenAmount);
        }
        Ok(())
    }

    /// Create master edition if it doesn't exist
    fn create_master_edition_if_needed(accounts: &MintUniversalNft<'info>) -> Result<()> {
        if accounts.master_edition.data_is_empty() {
            // At this point, supply must be 1
            require!(accounts.mint.supply == 1, Errors::InvalidTokenSupply);
            initialize_master_edition_account(
                &accounts.master_edition.to_account_info(),
                &accounts.mint.to_account_info(),
                &accounts.authority.to_account_info(),
                &accounts.payer.to_account_info(),
                &accounts.metadata.to_account_info(),
                &accounts.token_program.to_account_info(),
                &accounts.system_program.to_account_info(),
                &accounts.rent.to_account_info(),
                None,
            )?;
        }
        Ok(())
    }

    /// Create or validate origin PDA
    fn create_or_validate_origin_pda(
        accounts: &MintUniversalNft<'info>,
        nft_id: &[u8; 32],
        uri: &str,
        timestamp: i64,
    ) -> Result<()> {
        let (expected_origin, origin_bump) = Pubkey::find_program_address(&[b"universal_nft_origin", nft_id], &crate::id());
        require_keys_eq!(expected_origin, accounts.asset_origin.key(), Errors::InvalidProgram);

        if accounts.asset_origin.data_is_empty() {
            let rent = Rent::get()?;
            let space = UniversalNftOrigin::INIT_SPACE as u64;
            let lamports = rent.minimum_balance(space as usize);
            let origin_seeds: &[&[u8]] = &[b"universal_nft_origin", nft_id, &[origin_bump]];
            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    accounts.system_program.to_account_info(),
                    anchor_lang::system_program::CreateAccount { from: accounts.payer.to_account_info(), to: accounts.asset_origin.to_account_info() },
                    &[origin_seeds],
                ),
                lamports,
                space,
                &crate::id(),
            )?;

            // Write full account (discriminator + data) only on creation
            let mut data_ref = accounts.asset_origin.try_borrow_mut_data()?;
            let mut tmp = Vec::with_capacity(UniversalNftOrigin::INIT_SPACE);
            let origin_data = UniversalNftOrigin {
                nft_id: *nft_id,
                original_mint: accounts.mint.key(),
                original_metadata: accounts.metadata.key(),
                original_uri: uri.to_string(),
                is_on_solana: true,
                created_at: timestamp,
                bump_seed: origin_bump,
            };

            // Serialize full account (includes discriminator)
            anchor_lang::prelude::AccountSerialize::try_serialize(&origin_data, &mut tmp)
                .map_err(|_| Errors::InvalidDataFormat)?;
            require!(tmp.len() <= UniversalNftOrigin::INIT_SPACE, Errors::InvalidDataFormat);
            data_ref.fill(0);
            data_ref[..tmp.len()].copy_from_slice(&tmp);
        } else {
            // If already initialized and owned by this program, validate it matches expected values
            if *accounts.asset_origin.owner == crate::id() {
                let data_ref = accounts.asset_origin.try_borrow_data()?;
                require!(data_ref.len() >= 8, Errors::InvalidDataFormat);
                let mut bytes: &[u8] = &data_ref[8..]; // skip discriminator
                let existing = UniversalNftOrigin::try_deserialize(&mut bytes).map_err(|_| Errors::InvalidDataFormat)?;
                require!(existing.nft_id == *nft_id, Errors::OriginConflict);
                require!(existing.original_mint == accounts.mint.key(), Errors::OriginConflict);
                require!(existing.original_metadata == accounts.metadata.key(), Errors::OriginConflict);
                require!(existing.original_uri == uri, Errors::OriginConflict);
                // Matches expected; proceed without rewriting (idempotent)
            } else {
                return err!(Errors::InvalidAccountOwner);
            }
        }
        Ok(())
    }
}
