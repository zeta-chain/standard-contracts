use anchor_lang::prelude::*;
use crate::state::UniversalNftConfig;
use crate::errors::Errors;

#[derive(Accounts)]
pub struct ModifySettings<'info> {
    #[account(mut)]
    pub administrator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.pda_bump,
        constraint = config.admin.is_some(),
        constraint = config.admin.unwrap() == administrator.key()
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    pub system_program: Program<'info, System>,
}

impl ModifySettings<'_> {
    pub fn modify_program_settings(
        ctx: Context<Self>,
        new_admin: Option<Pubkey>,
        new_gateway_id: Option<Pubkey>,
        new_verifier: Option<Pubkey>,
        pause_state: Option<bool>,
    ) -> Result<()> {
        let settings = &mut ctx.accounts.config;
        
        let previous_admin = settings.admin;
        let previous_gateway = settings.zeta_gateway_program_id;
        let previous_verifier = settings.zeta_gateway_verifier;
        
        if let Some(admin_pubkey) = new_admin {
            require!(
                admin_pubkey != Pubkey::default(),
                Errors::InvalidParameter
            );
            settings.admin = Some(admin_pubkey);
        }
        
        if let Some(gateway_pubkey) = new_gateway_id {
            require!(
                gateway_pubkey != Pubkey::default(),
                Errors::InvalidParameter
            );
            
            settings.zeta_gateway_program_id = gateway_pubkey;
        }
        
        if let Some(verifier_pubkey) = new_verifier {
            require!(
                verifier_pubkey != Pubkey::default(),
                Errors::InvalidParameter
            );
            
            settings.zeta_gateway_verifier = verifier_pubkey;
        }
        
        if let Some(should_pause) = pause_state {
            settings.paused = should_pause;
        }
        
        msg!(
            "Program settings successfully modified\nPrevious admin: {:?}\nNew admin: {:?}\nPrevious gateway: {}\nNew gateway: {}\nPrevious verifier: {}\nNew verifier: {}\nModified by: {}\nModification time: {}",
            previous_admin,
            settings.admin,
            previous_gateway,
            settings.zeta_gateway_program_id,
            previous_verifier,
            settings.zeta_gateway_verifier,
            ctx.accounts.administrator.key(),
            Clock::get()?.unix_timestamp
        );
        
        Ok(())
    }
}

