use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_lang::prelude::UncheckedAccount;
use sha2::{Digest, Sha256};

use crate::state::nft_origin::NftOrigin;
use crate::utils::*;

#[derive(Accounts)]
pub struct MintNewNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub recipient: SystemAccount<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_authority_pda,
        mint::freeze_authority = mint_authority_pda,
    )]
    pub mint: Account<'info, Mint>,
    /// CHECK: Metaplex metadata PDA for this mint; created via CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: Metaplex master edition PDA for this mint; created via CPI
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    /// CHECK: nft_origin PDA; will be created programmatically with seeds
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: mint_authority PDA; will be derived programmatically
    pub mint_authority_pda: UncheckedAccount<'info>,
    /// CHECK: token metadata program (Metaplex)
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<MintNewNft>, metadata_uri: String) -> Result<()> {
    let clock = Clock::get()?;

    require!(metadata_uri.len() <= NftOrigin::MAX_URI_LEN, ErrorCode::MetadataTooLong);

    // Generate unique token ID: hash of mint pubkey + slot + timestamp
    let mut hasher = Sha256::new();
    hasher.update(ctx.accounts.mint.key().as_ref());
    hasher.update(&clock.slot.to_le_bytes());
    hasher.update(&clock.unix_timestamp.to_le_bytes());
    let token_id_hash = hasher.finalize();
    let mut token_id: [u8; 32] = [0u8; 32];
    token_id.copy_from_slice(&token_id_hash[..32]);

    // Derive mint authority PDA
    let (mint_authority_pda, mint_authority_bump) = Pubkey::find_program_address(
        &[b"mint_authority", ctx.accounts.mint.key().as_ref()],
        &crate::ID,
    );
    require_keys_eq!(ctx.accounts.mint_authority_pda.key(), mint_authority_pda, ErrorCode::InvalidMintAuthorityPda);

    // Validate metadata and master edition PDAs
    let (expected_metadata_pda, _) = derive_metadata_pda(&ctx.accounts.mint.key());
    let (expected_master_edition_pda, _) = derive_master_edition_pda(&ctx.accounts.mint.key());
    require_keys_eq!(ctx.accounts.metadata.key(), expected_metadata_pda, ErrorCode::InvalidMetadataPda);
    require_keys_eq!(ctx.accounts.master_edition.key(), expected_master_edition_pda, ErrorCode::InvalidMasterEditionPda);

    // Mint 1 token to recipient
    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority_pda.to_account_info(),
            },
            &[&[b"mint_authority", ctx.accounts.mint.key().as_ref(), &[mint_authority_bump]]],
        ),
        1,
    )?;

    // Create Metaplex metadata and master edition
    cpi_create_metadata_v3(
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.mint_authority_pda.to_account_info(),
        &ctx.accounts.mint_authority_pda.to_account_info(),
        &ctx.accounts.metadata.to_account_info(),
        &ctx.accounts.token_metadata_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        "UniversalNFT".to_string(),
        "UNFT".to_string(),
        metadata_uri.clone(),
    )?;

    cpi_create_master_edition_v3(
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.mint_authority_pda.to_account_info(),
        &ctx.accounts.mint_authority_pda.to_account_info(),
        &ctx.accounts.metadata.to_account_info(),
        &ctx.accounts.master_edition.to_account_info(),
        &ctx.accounts.token_metadata_program.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
    )?;

    // Create nft_origin PDA deterministically
    let (nft_origin_pda, nft_origin_bump) = derive_nft_origin_pda(&token_id);
    require_keys_eq!(ctx.accounts.nft_origin.key(), nft_origin_pda, ErrorCode::NftOriginPdaMismatch);
    
    if ctx.accounts.nft_origin.data_is_empty() {
        let space = 8 + NftOrigin::LEN; // add discriminator
        let lamports = Rent::get()?.minimum_balance(space);
        anchor_lang::solana_program::program::invoke_signed(
            &anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.payer.key(),
                &nft_origin_pda,
                lamports,
                space as u64,
                &crate::ID,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.nft_origin.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[&token_id, b"nft_origin", &[nft_origin_bump]]],
        )?;
    }
    
    let nft_origin = NftOrigin {
        origin_chain: 0u64, // Solana
        origin_token_id: token_id,
        origin_mint: ctx.accounts.mint.key(),
        metadata_uri,
        created_at: clock.unix_timestamp,
        bump: nft_origin_bump,
    };

    // Write discriminator + data
    use anchor_lang::Discriminator;
    let mut data = ctx.accounts.nft_origin.try_borrow_mut_data()?;
    data[..8].copy_from_slice(&NftOrigin::discriminator());
    nft_origin.try_serialize(&mut &mut data[8..])?;

    msg!("Minted Universal NFT with token ID: {}", hex::encode(&token_id[..8]));
    msg!("Metadata URI: {}", nft_origin.metadata_uri);
    msg!("NFT Origin PDA: {}", nft_origin_pda);

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Metadata URI too long")]
    MetadataTooLong,
    #[msg("Invalid Metadata PDA")]
    InvalidMetadataPda,
    #[msg("Invalid Master Edition PDA")]
    InvalidMasterEditionPda,
    #[msg("Invalid NftOrigin PDA")]
    NftOriginPdaMismatch,
    #[msg("Invalid Mint Authority PDA")]
    InvalidMintAuthorityPda,
}
