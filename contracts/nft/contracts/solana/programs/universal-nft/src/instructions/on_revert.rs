use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::sysvar::rent::Rent;
use anchor_lang::system_program::{Transfer, transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::{Collection, NftOrigin};
use crate::{TokenTransferReverted, UniversalNftError, ZETACHAIN_GATEWAY_PROGRAM_ID, TOKEN_METADATA_PROGRAM_ID, GATEWAY_PDA_SEED};

/// Handle failed cross-chain transfers by minting NFT back to original sender
pub fn on_revert(
    ctx: Context<OnRevertContext>,
    token_id: u64,
    uri: String,
    original_sender: Vec<u8>,
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
    
    // Validate original_sender account
    require!(
        ctx.accounts.original_sender.key() != anchor_lang::solana_program::system_program::ID,
        UniversalNftError::InvalidRecipient
    );
    require!(
        !ctx.accounts.original_sender.executable,
        UniversalNftError::InvalidRecipient
    );
    require!(
        ctx.accounts.original_sender.is_writable,
        UniversalNftError::InvalidRecipient
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
        "Reverted NFT transfer - minted token_id {} back to original sender {:?}",
        token_id,
        original_sender
    );
    
    // Emit event
    emit!(TokenTransferReverted {
        collection: collection_key,
        token_id,
        sender: {
            if original_sender.len() == 32 {
                Pubkey::new_from_array(original_sender.try_into().unwrap_or_default())
            } else {
                Pubkey::default()
            }
        },
        uri,
        refund_amount,
        origin_chain: 0,
        original_mint: None,
        revert_reason: "Cross-chain transfer reverted".to_string(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct OnRevertContext<'info> {
    #[account(
        mut,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump
    )]
    pub collection: Account<'info, Collection>,
    
    pub collection_mint: Account<'info, Mint>,
    
    /// CHECK: Gateway program that calls this function
    #[account(address = ZETACHAIN_GATEWAY_PROGRAM_ID)]
    pub gateway: UncheckedAccount<'info>,
    
    /// CHECK: Gateway PDA account
    #[account(
        seeds = [GATEWAY_PDA_SEED],
        bump
    )]
    pub gateway_pda: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"nft_origin", &token_id.to_le_bytes()[..]],
        bump,
        space = 8 + NftOrigin::INIT_SPACE
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
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
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: mpl-token-metadata program
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}
