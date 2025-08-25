use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
};
use crate::state::{UniversalNftConfig, NftOrigin};
use crate::util::constants::{*, TOKEN_METADATA_PROGRAM_ID};

#[derive(Accounts)]
#[instruction(token_id: [u8; 32])]
pub struct TransferToZetachain<'info> {
    #[account(
        mut,
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    #[account(
        mut,
        seeds = [NFT_ORIGIN_SEED, token_id.as_ref()],
        bump = nft_origin.bump
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    #[account(
        mut,
        constraint = mint.key() == nft_origin.original_mint @ crate::error::UniversalNftError::InvalidMint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
        constraint = token_account.amount == 1 @ crate::error::UniversalNftError::InvalidTokenAmount
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// Validated metadata account
    /// CHECK: This account is validated to match the original metadata account stored in nft_origin
    #[account(
        mut,
        constraint = metadata.key() == nft_origin.original_metadata @ crate::error::UniversalNftError::InvalidMetadata,
        seeds = [
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = TOKEN_METADATA_PROGRAM_ID
    )]
    pub metadata: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// ZetaChain Gateway Program
    /// CHECK: This is the ZetaChain gateway program for cross-chain operations
    #[account(
        constraint = gateway_program.key() == config.gateway_program @ crate::error::UniversalNftError::InvalidGatewayProgram,
        executable
    )]
    pub gateway_program: UncheckedAccount<'info>,
    
    /// ZetaChain Gateway PDA
    /// CHECK: This is the PDA for the ZetaChain gateway; enforce ownership by gateway program
    #[account(
        mut,
        constraint = *gateway_pda.owner == config.gateway_program @ crate::error::UniversalNftError::InvalidGatewayProgram,
        constraint = gateway_pda.key() == config.gateway_pda @ crate::error::UniversalNftError::InvalidGatewayProgram
    )]
    pub gateway_pda: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
