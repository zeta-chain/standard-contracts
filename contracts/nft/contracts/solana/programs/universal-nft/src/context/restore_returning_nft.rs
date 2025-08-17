use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::state::{UniversalNftConfig, NftOrigin};
use crate::util::constants::{*, TOKEN_METADATA_PROGRAM_ID};

#[derive(Accounts)]
#[instruction(token_id: [u8; 32])]
pub struct RestoreReturningNft<'info> {
    /// Program configuration and authority
    #[account(
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    /// NFT origin account - must exist for returning NFTs
    #[account(
        mut,
        seeds = [NFT_ORIGIN_SEED, token_id.as_ref()],
        bump = nft_origin.bump
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    /// New mint account for the restored NFT (since original was burned)
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = config,
        mint::freeze_authority = config,
    )]
    pub mint: Account<'info, Mint>,
    
    /// Metadata account derived from mint
    /// CHECK: This account is derived from the mint using seeds, ensuring it's the correct metadata account
    #[account(
        mut,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,
    
    /// Master edition account derived from mint
    /// CHECK: This account is derived from the mint using seeds, ensuring it's the correct master edition account
    #[account(
        mut,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
            b"edition",
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub master_edition: UncheckedAccount<'info>,
    
    /// Token account for the recipient
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// Payer for account creation fees
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Recipient of the restored NFT
    #[account(
        constraint = recipient.key() != Pubkey::default() @ crate::error::UniversalNftError::InvalidRecipientAddress
    )]
    pub recipient: SystemAccount<'info>,
    
    /// Token program for NFT operations
    pub token_program: Program<'info, Token>,
    
    /// Associated token program for creating token accounts
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// Metaplex Token Metadata program
    /// CHECK: This is verified to be the correct Metaplex Token Metadata program ID
    #[account(
        constraint = metadata_program.key() == TOKEN_METADATA_PROGRAM_ID @ crate::error::UniversalNftError::InvalidProgram
    )]
    pub metadata_program: UncheckedAccount<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
    
    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}
