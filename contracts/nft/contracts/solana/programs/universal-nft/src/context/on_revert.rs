use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;

use crate::state::*;
use crate::util::constants::*;

/// Context for handling reverted cross-chain transactions
/// Called by ZetaChain gateway when a transaction fails and needs to be reverted
#[derive(Accounts)]
pub struct OnRevert<'info> {
    /// Program configuration and authority
    #[account(
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump,
    )]
    pub config: Account<'info, UniversalNftConfig>,

    /// NFT origin tracking (may exist if restoring a burned NFT)
    #[account(
        mut,
        seeds = [NFT_ORIGIN_SEED, nft_origin.token_id.as_ref()],
        bump = nft_origin.bump,
    )]
    pub nft_origin: Account<'info, NftOrigin>,

    /// New mint account for restored NFT (if needed)
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = config,
    )]
    pub mint: Account<'info, Mint>,

    /// Token account for the restored NFT
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// Recipient who will receive the restored NFT
    /// CHECK: This is validated as the original owner in the revert logic
    pub recipient: UncheckedAccount<'info>,

    /// Metadata account for the restored NFT
    /// CHECK: This account is initialized via CPI to Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Master edition account for the restored NFT
    /// CHECK: This account is initialized via CPI to Metaplex
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// Payer for account creation fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Instructions sysvar for verifying the caller
    /// CHECK: This is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar_account: UncheckedAccount<'info>,

    /// Metaplex Token Metadata program
    /// CHECK: This is the Metaplex Token Metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}
