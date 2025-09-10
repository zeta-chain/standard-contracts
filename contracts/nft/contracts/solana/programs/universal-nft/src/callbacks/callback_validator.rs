use anchor_lang::prelude::*;
use crate::errors::Errors;

pub struct CallbackValidator;

impl CallbackValidator {
    pub fn verify_gateway_origin(
        remaining_accounts: &[AccountInfo],
        expected_gateway_program: Pubkey,
    ) -> Result<()> {
        let ix_sysvar = anchor_lang::solana_program::sysvar::instructions::id();
        
        let ix_account = remaining_accounts
            .iter()
            .find(|ai| ai.key() == ix_sysvar)
            .ok_or(Errors::InvalidCaller)?;
        
        let current_ix = anchor_lang::solana_program::sysvar::instructions::get_instruction_relative(
            -1,
            ix_account,
        ).map_err(|_| Errors::InvalidCaller)?;
        
        require!(
            current_ix.program_id == expected_gateway_program,
            Errors::InvalidCaller
        );

        Ok(())
    }

    pub fn validate_token_account(
        token_account: &AccountInfo,
        expected_owner: Pubkey,
        expected_mint: Pubkey,
    ) -> Result<()> {
        require_keys_eq!(*token_account.owner, expected_owner, Errors::InvalidProgram);
        
        let expected_ata = anchor_spl::associated_token::get_associated_token_address(
            &expected_owner,
            &expected_mint,
        );

        require_keys_eq!(
            expected_ata,
            token_account.key(),
            Errors::InvalidProgram
        );

        Ok(())
    }

    pub fn validate_asset_tracker_pda(
        asset_tracker: &AccountInfo,
        asset_identifier: [u8; 32],
        program_id: Pubkey,
    ) -> Result<()> {
        let (expected_tracker_key, _) = Pubkey::find_program_address(
            &[b"asset_tracker", asset_identifier.as_ref()],
            &program_id,
        );
        
        require_keys_eq!(
            expected_tracker_key,
            asset_tracker.key(),
            Errors::InvalidDataFormat
        );

        Ok(())
    }
}
