
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn};
use mpl_token_metadata::instruction::{create_metadata_accounts_v3};
use anchor_lang::solana_program::program::invoke_signed;

declare_id!("NFTBridge11111111111111111111111111111111111");

#[program]
pub mod zetamint {
    use super::*;

    pub fn mint_nft(ctx: Context<MintNFT>, uri: String, title: String, symbol: String) -> Result<()> {
        // Mint token to user's associated account
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let signer_seeds = &[b"mint_authority", &[*ctx.bumps.get("mint_authority").unwrap()]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[&signer_seeds[..]]);
        token::mint_to(cpi_ctx, 1)?;

        // Create metadata account
        let metadata_accounts = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let metadata_instruction = create_metadata_accounts_v3(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.mint_authority.key(),
            title,
            symbol,
            uri,
            None,
            1,
            true,
            false,
            None,
            None,
            None,
        );

        invoke_signed(
            &metadata_instruction,
            metadata_accounts.as_slice(),
            &[&signer_seeds[..]],
        )?;

        Ok(())
    }

    pub fn burn_nft(ctx: Context<BurnNFT>) -> Result<()> {
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, 1)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(uri: String, title: String, symbol: String)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, mint::decimals = 0, mint::authority = mint_authority)]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK: Checked via Metaplex
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: Metaplex
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
