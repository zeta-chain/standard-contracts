use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Mint, Token, TokenAccount};

/// Generate a single token unit for the specified recipient
/// This operation is used during Universal NFT creation process
pub fn generate_token_unit_for_recipient<'info>(
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
