use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::UniversalNftConfig;
use crate::util::constants::{UNIVERSAL_NFT_CONFIG_SEED, TOKEN_METADATA_PROGRAM_ID};

#[derive(Accounts)]
pub struct OnCall<'info> {
    /// Program configuration and authority
    #[account(
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = pda.bump
    )]
    pub pda: Account<'info, UniversalNftConfig>,
    
    /// NFT origin account (PDA) derived as [NFT_ORIGIN_SEED, origin_chain.to_be_bytes(), token_id]
    /// CHECK: Passed PDA must match derived address; we (re)initialize if empty
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,

    /// Program ATA to receive minted token (authority = config, mint = mint)
    /// We validate at runtime that it matches the derived ATA
    #[account(mut)]
    pub pda_ata: Account<'info, TokenAccount>,

    /// Mint account of the NFT (must have authority = config, decimals = 0)
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,

    /// Metadata account derived from mint - validated for security
    /// CHECK: PDA derived via Metaplex seeds
    #[account(
        mut,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint_account.key().as_ref(),
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,

    /// Master edition account derived from mint - validated for security
    /// CHECK: PDA derived via Metaplex seeds
    #[account(
        mut,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint_account.key().as_ref(),
            b"edition",
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// Gateway PDA, used for minimal caller verification
    /// CHECK: Read-only; enforce that the PDA equals stored gateway_pda and is owned by the configured gateway program
    #[account(
        constraint = gateway_pda.key() == pda.gateway_pda @ crate::error::UniversalNftError::InvalidGatewayProgram,
        constraint = *gateway_pda.owner == pda.gateway_program @ crate::error::UniversalNftError::InvalidGatewayProgram
    )]
    pub gateway_pda: UncheckedAccount<'info>,

    /// CHECK: Verified against constant ID
    #[account(constraint = metadata_program.key() == TOKEN_METADATA_PROGRAM_ID @ anchor_lang::error::ErrorCode::ConstraintAddress)]
    pub metadata_program: UncheckedAccount<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,

    /// Token program for NFT operations
    pub token_program: Program<'info, Token>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}