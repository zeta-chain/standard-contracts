use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::{
    token::Token,
    associated_token::AssociatedToken,
};
use crate::state::UniversalNftConfig;
use crate::util::constants::{UNIVERSAL_NFT_CONFIG_SEED, TOKEN_METADATA_PROGRAM_ID};

#[derive(Accounts)]
pub struct OnCall<'info> {
    /// Program configuration and authority
    #[account(
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    /// NFT origin account - may or may not exist depending on whether NFT was previously on Solana
    /// CHECK: This is the NFT origin PDA, we check manually if it's initialized
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,
    
    /// Payer for account creation fees
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Instruction sysvar for caller verification
    /// CHECK: This is the instruction sysvar account for verifying the calling program
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
    
    /// Token program for NFT operations
    pub token_program: Program<'info, Token>,
    
    /// Associated token program for creating token accounts
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// Metaplex Token Metadata program
    /// CHECK: This is the Metaplex Token Metadata program address
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
    
    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}