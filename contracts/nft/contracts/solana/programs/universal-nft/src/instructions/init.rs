use anchor_lang::prelude::*;

use crate::{
    errors::Errors,
    state::UniversalNftConfig,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Config PDA account to initialize
    #[account(
        init,
        payer = admin,
        space = 8 + UniversalNftConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, UniversalNftConfig>,

    /// Gateway program account passed in
    /// This is used for validation (executable + matches the provided ID)
    pub gateway_program: UncheckedAccount<'info>,

    /// Some PDA owned by the gateway program
    pub gateway_pda: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(ctx: Context<Self>, zeta_gateway_program_id: Pubkey) -> Result<()> {
    let gateway_program = &ctx.accounts.gateway_program;
    let gateway_pda = &ctx.accounts.gateway_pda;

    // Validation checks with granular errors
    require!(
        zeta_gateway_program_id != Pubkey::default(),
        Errors::GatewayProgramDefault
    );
    require!(gateway_program.executable, Errors::GatewayProgramNotExecutable);
    require!(
        gateway_pda.owner == &zeta_gateway_program_id,
        Errors::GatewayPdaOwnershipInvalid
    );
    require_keys_eq!(
        gateway_program.key(),
        zeta_gateway_program_id,
        Errors::GatewayProgramMismatch
    );

    // Initialize config
    let config = &mut ctx.accounts.config;
    let clock = Clock::get()?;

    **config = UniversalNftConfig {
        admin: Some(ctx.accounts.admin.key()),
        zeta_gateway_program_id,
        zeta_gateway_verifier: ctx.accounts.gateway_pda.key(),
        message_sequence: 0,
        next_nft_id: 0,
        paused: false,
        initialized_timestamp: clock.unix_timestamp,
        pda_bump: ctx.bumps.config,
    };

    // Log initialization event
    if let Some(admin_key) = config.admin {
        msg!("Universal NFT program initialized. Admin authority set to {}", admin_key);
    } else {
        msg!("Universal NFT program initialized with no admin authority set.");
    }

    Ok(())
    }
}
