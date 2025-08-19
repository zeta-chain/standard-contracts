use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::state::{UniversalNftConfig, MintTicket};
use crate::util::constants::{*, TOKEN_METADATA_PROGRAM_ID, MINT_TICKET_SEED};

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(
        mut,
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    /// Reservation ticket created via ReserveNextTokenId; consumed here
    #[account(
        mut,
        seeds = [MINT_TICKET_SEED, mint.key().as_ref(), authority.key().as_ref()],
        bump = ticket.bump,
        constraint = !ticket.used @ crate::error::UniversalNftError::OperationNotAllowed,
        has_one = authority,
    )]
    pub ticket: Account<'info, MintTicket>,

    /// Origin PDA passed but initialized in the handler using program-generated token_id.
    /// The handler recomputes token_id from ticket.{slot,reserved_id} and requires PDA match.
    /// CHECK: Verified in handler
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,
    
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
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,
    
    /// Master edition account derived from mint - validated for security
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
        constraint = recipient.key() != Pubkey::default() @ crate::error::UniversalNftError::InvalidRecipientAddress
    )]
    pub recipient: SystemAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Validated Metaplex Token Metadata program
    /// CHECK: This is verified to be the correct Metaplex Token Metadata program ID
    #[account(
        constraint = metadata_program.key() == TOKEN_METADATA_PROGRAM_ID @ crate::error::UniversalNftError::InvalidProgram
    )]
    pub metadata_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
