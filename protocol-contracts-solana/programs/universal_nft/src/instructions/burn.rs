use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::nft_origin::{NftOrigin, CrossChainNftPayload};
use crate::state::replay::ReplayMarker;
use crate::utils::derive_replay_marker_pda;

#[derive(Accounts)]
pub struct BurnForTransfer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub nft_origin: Account<'info, NftOrigin>,
    /// CHECK: created/validated in handler via seeds
    #[account(mut)]
    pub replay_marker: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BurnForTransfer>, nonce: u64) -> Result<()> {
    let clock = Clock::get()?;

    // Ensure ATA matches mint and owner
    require_keys_eq!(ctx.accounts.owner_token_account.owner, ctx.accounts.owner.key(), ErrorCode::UnauthorizedOwner);
    require_keys_eq!(ctx.accounts.owner_token_account.mint, ctx.accounts.mint.key(), ErrorCode::InvalidMint);
    require!(ctx.accounts.owner_token_account.amount > 0, ErrorCode::NoTokensToBurn);

    // Derive replay marker PDA
    let (replay_pda, bump) = derive_replay_marker_pda(&ctx.accounts.nft_origin.origin_token_id, nonce);
    require_keys_eq!(ctx.accounts.replay_marker.key(), replay_pda, ErrorCode::ReplayPdaMismatch);

    // If not initialized, create replay marker
    if ctx.accounts.replay_marker.data_is_empty() {
        let space = 8 + ReplayMarker::LEN;
        let lamports = Rent::get()?.minimum_balance(space);
        anchor_lang::solana_program::program::invoke_signed(
            &anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.owner.key(),
                &replay_pda,
                lamports,
                space as u64,
                &crate::ID,
            ),
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.replay_marker.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[ReplayMarker::SEED, &ctx.accounts.nft_origin.origin_token_id, &nonce.to_le_bytes(), &[bump]]],
        )?;

        // Write marker
        let marker = ReplayMarker { token_id: ctx.accounts.nft_origin.origin_token_id, nonce, created_at: clock.unix_timestamp, bump };
        let mut data = ctx.accounts.replay_marker.try_borrow_mut_data()?;
        marker.try_serialize(&mut &mut data[..])?;
    } else {
        return Err(ErrorCode::ReplayAttempt.into());
    }

    // Burn 1 token
    anchor_spl::token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.owner_token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        1,
    )?;

    // Emit cross-chain payload event
    let payload = CrossChainNftPayload {
        version: 1,
        token_id: ctx.accounts.nft_origin.origin_token_id,
        origin_chain_id: ctx.accounts.nft_origin.origin_chain,
        origin_mint: ctx.accounts.nft_origin.origin_mint,
        metadata_uri: ctx.accounts.nft_origin.metadata_uri.clone(),
        recipient: ctx.accounts.owner.key(),
        nonce,
    };

    emit!(CrossChainTransferEvent {
        token_id: payload.token_id,
        origin_chain: payload.origin_chain_id,
        destination_chain: 0,
        owner: payload.recipient,
        nonce: payload.nonce,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct CrossChainTransferEvent {
    pub token_id: [u8; 32],
    pub origin_chain: u16,
    pub destination_chain: u16,
    pub owner: Pubkey,
    pub nonce: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized owner")] UnauthorizedOwner,
    #[msg("Invalid mint")] InvalidMint,
    #[msg("No tokens to burn")] NoTokensToBurn,
    #[msg("Replay attempt detected")] ReplayAttempt,
    #[msg("Replay PDA mismatch")] ReplayPdaMismatch,
}
