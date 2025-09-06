use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::Collection;
use crate::{TokenTransferReverted, UniversalNftError, ZETACHAIN_GATEWAY_PROGRAM_ID, TOKEN_METADATA_PROGRAM_ID};

/// Handle failed cross-chain transfers by minting NFT back to original sender
pub fn on_revert(
    ctx: Context<OnRevertContext>,
    token_id: u64,
    uri: String,
    original_sender: Pubkey,
    refund_amount: u64,
) -> Result<()> {

    // Verify the caller is the gateway program
    require!(
        ctx.accounts.gateway.key() == ZETACHAIN_GATEWAY_PROGRAM_ID,
        UniversalNftError::UnauthorizedGateway
    );
    
    // Get collection data before mutable borrow
    let collection = &ctx.accounts.collection;
    let collection_authority = collection.authority;
    let collection_name = collection.name.clone();
    let collection_bump = collection.bump;
    let collection_key = collection.key();
    
    // Mint the NFT back to the original sender
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.nft_mint.to_account_info(),
            to: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.collection.to_account_info(),
        },
    );
    
    // Use collection as signer
    let seeds = &[
        b"collection",
        collection_authority.as_ref(),
        collection_name.as_bytes(),
        &[collection_bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    mint_to(cpi_ctx.with_signer(signer_seeds), 1)?;
    
    // Handle refund if applicable
    if refund_amount > 0 {
        // Transfer SOL refund to original sender
        let lamports = ctx.accounts.payer.lamports();
        if lamports >= refund_amount {
            **ctx.accounts.payer.lamports.borrow_mut() -= refund_amount;
            **ctx.accounts.original_sender.lamports.borrow_mut() += refund_amount;
        }
    }
    
    msg!(
        "Reverted NFT transfer - minted token_id {} back to original sender {}",
        token_id,
        original_sender
    );
    
    // Emit event
    emit!(TokenTransferReverted {
        collection: collection_key,
        token_id,
        sender: original_sender,
        uri,
        refund_amount,
        origin_chain: None,
        original_mint: None,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct OnRevertContext<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Gateway program that calls this function
    #[account(address = ZETACHAIN_GATEWAY_PROGRAM_ID)]
    pub gateway: UncheckedAccount<'info>,
    
    /// CHECK: Gateway PDA account
    pub gateway_pda: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = original_sender,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Original sender account to receive the reverted NFT
    pub original_sender: UncheckedAccount<'info>,
    
    /// CHECK: Metadata account for the NFT
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}