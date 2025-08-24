use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::Hasher;

use crate::{
    errors::Errors,
    state::{UniversalNftConfig, TokenReservation},
};

#[derive(Accounts)]
pub struct AllocateTokenId<'info> {
    #[account(
        mut,    
        seeds = [b"config"],
        bump = config.pda_bump
    )]
    pub config: Account<'info, UniversalNftConfig>,

    #[account(
        init,
        payer = admin,
        space = 8 + TokenReservation::INIT_SPACE,
        seeds = [
            b"token_reservation",
            mint.key().as_ref(),
            admin.key().as_ref()
        ],
        bump
    )]
    pub reservation: Account<'info, TokenReservation>,

    /// CHECK: Only used as seed for PDA derivation
    pub mint: UncheckedAccount<'info>,

    /// Admin authority that can create reservations
    #[account(mut)]
    pub admin: Signer<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

impl<'info> AllocateTokenId<'info> {
    pub fn allocate_token_identifier(ctx: Context<Self>) -> Result<()> {
        require!(!ctx.accounts.config.paused, Errors::ProgramPaused);

        require!(
            ctx.accounts.config.admin == Some(ctx.accounts.admin.key()),
            Errors::GatewayProgramDefault
        );

        let current_slot = Clock::get()?.slot;
        let current_token_id = ctx.accounts.config.next_nft_id;

        let mut hash_calculator = Hasher::default();
        hash_calculator.hash(ctx.accounts.mint.key().as_ref());
        hash_calculator.hash(&current_slot.to_le_bytes());
        hash_calculator.hash(&current_token_id.to_le_bytes());
        let computed_hash = hash_calculator.result().to_bytes();

        let reservation = &mut ctx.accounts.reservation;
        reservation.mint_address = ctx.accounts.mint.key();
        reservation.creator = ctx.accounts.admin.key();
        reservation.reserved_id = current_token_id;
        reservation.block_slot = current_slot;
        reservation.token_hash = computed_hash;
        reservation.is_consumed = false;
        reservation.creation_time = Clock::get()?.unix_timestamp;
        reservation.bump_seed = ctx.bumps.reservation;

        ctx.accounts.config.next_nft_id = ctx.accounts.config
            .next_nft_id
            .checked_add(1)
            .ok_or(Errors::MathOverflow)?;

        msg!("Token ID {} reserved for mint {}", current_token_id, ctx.accounts.mint.key());
        
        Ok(())
    }
}
