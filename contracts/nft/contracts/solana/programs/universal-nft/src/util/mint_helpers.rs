use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Mint, Token, TokenAccount};

/// Common NFT minting utilities for use across mint_nft, on_call
/// These functions handle the core NFT creation and minting logic

/// Mint an NFT token to a recipient's token account
/// This is a common operation used in mint_nft, on_call
pub fn mint_nft_to_recipient<'info>(
    mint_account: &Account<'info, Mint>,
    token_account: &Account<'info, TokenAccount>,
    authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    authority_signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let mint_to_ctx = if let Some(signer_seeds) = authority_signer_seeds {
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                mint: mint_account.to_account_info(),
                to: token_account.to_account_info(),
                authority: authority.clone(),
            },
            signer_seeds,
        )
    } else {
        CpiContext::new(
            token_program.to_account_info(),
            MintTo {
                mint: mint_account.to_account_info(),
                to: token_account.to_account_info(),
                authority: authority.clone(),
            },
        )
    };
    
    token::mint_to(mint_to_ctx, 1)?;
    Ok(())
}