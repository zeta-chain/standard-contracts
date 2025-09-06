use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::Collection;
use crate::{TokenTransferReverted, UniversalNftError, ZETACHAIN_GATEWAY_PROGRAM_ID, TOKEN_METADATA_PROGRAM_ID, GATEWAY_PDA_SEED};

/// Handle failed cross-chain transfers by minting NFT back to original sender
pub fn on_revert(
    ctx: Context<OnRevertContext>,
    token_id: u64,
    uri: String,
    original_sender: Pubkey,
    refund_amount: u64,
) -> Result<()> {

    // Verify the caller is the gateway program and validate PDA
    require!(
        ctx.accounts.gateway.key() == ZETACHAIN_GATEWAY_PROGRAM_ID,
        UniversalNftError::UnauthorizedGateway
    );
    
    // Derive expected gateway PDA and verify it matches
    let (expected_gateway_pda, _) = Pubkey::find_program_address(
        &[GATEWAY_PDA_SEED],
        &ZETACHAIN_GATEWAY_PROGRAM_ID,
    );
    require!(
        ctx.accounts.gateway_pda.key() == expected_gateway_pda,
        UniversalNftError::UnauthorizedGateway
    );
    
    // Verify gateway PDA is owned by the gateway program
    require!(
        ctx.accounts.gateway_pda.owner == &ZETACHAIN_GATEWAY_PROGRAM_ID,
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
    
    // Handle refund if applicable using System Program CPI
    if refund_amount > 0 {
        // Get rent to ensure payer remains rent-exempt
        let rent = Rent::get()?;
        let payer_account_data_len = ctx.accounts.payer.to_account_info().data_len();
        let minimum_balance = rent.minimum_balance(payer_account_data_len);
        
        // Assert payer has sufficient balance and will remain rent-exempt
        require!(
            ctx.accounts.payer.lamports() >= minimum_balance + refund_amount,
            UniversalNftError::InsufficientGasAmount
        );
        
        // Construct CPI context for system program transfer
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.original_sender.to_account_info(),
            },
        );
        
        // Execute the transfer via System Program CPI
        transfer(cpi_ctx, refund_amount)?;
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