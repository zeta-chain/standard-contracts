use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
};
use crate::{
    state::{UniversalNftConfig, UniversalNftOrigin},
    errors::Errors,
    util::{bridge_constants::*, inter_chain_helpers, bridge_operations},
};

#[derive(Accounts)]
#[instruction(asset_identifier: [u8; 32])]
pub struct CrossChainBridge<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = settings.pda_bump
    )]
    pub settings: Account<'info, UniversalNftConfig>,
    
    #[account(
        mut,
        seeds = [b"asset_tracker", asset_identifier.as_ref()],
        bump = asset_tracker.bump_seed
    )]
    pub asset_tracker: Account<'info, UniversalNftOrigin>,
    
    #[account(
        mut,
        constraint = mint.key() == asset_tracker.original_mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = asset_owner,
        constraint = token_account.amount == 1 @ Errors::InvalidTokenAmount
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub asset_owner: Signer<'info>,
    
    /// Cross-Chain Bridge Program
    /// CHECK: This is the cross-chain bridge program for inter-chain operations
    #[account(
        constraint = bridge_program.key() == settings.zeta_gateway_program_id,
        executable
    )]
    pub bridge_program: UncheckedAccount<'info>,
    
    /// Cross-Chain Bridge PDA
    /// CHECK: This is the PDA for the cross-chain bridge; enforce ownership by bridge program
    #[account(
        mut,
        constraint = *bridge_pda.owner == settings.zeta_gateway_program_id,
        constraint = bridge_pda.key() == settings.zeta_gateway_verifier
    )]
    pub bridge_pda: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CrossChainBridge<'info> {
    pub fn bridge_to_zetachain(
        ctx: Context<Self>,
        asset_identifier: [u8; 32],
        zetachain_universal_contract: [u8; 20],
        final_destination_chain: u64,
        final_recipient: String,
        sol_deposit_lamports: u64,
    ) -> Result<()> {
        // Validate program state and parameters
        ctx.accounts.validate_program_state(&final_recipient, sol_deposit_lamports)?;
        
        // Validate bridge configuration
        ctx.accounts.validate_bridge_configuration()?;
        
        // Validate asset tracker
        ctx.accounts.validate_asset_tracker(asset_identifier)?;
        
        // Validate token account
        ctx.accounts.validate_token_account()?;
        
        // Validate mint for burning
        ctx.accounts.validate_mint_for_burning()?;
        
        // Validate recipient address
        let receiver_addr = ctx.accounts.validate_recipient_address(&final_recipient)?;

        let clock = Clock::get()?;
        
        // Burn the asset on Solana
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.asset_owner.to_account_info(),
            },
        );
        anchor_spl::token::burn(burn_ctx, 1)?;
        
        // Update origin tracking
        let asset_tracker = &mut ctx.accounts.asset_tracker;
        asset_tracker.mark_transferred_off_solana(clock.unix_timestamp);
        
        let uri = asset_tracker.original_uri.clone();
        let sender_addr: [u8; 20] = [0u8; 20];
        let cross_chain_message = bridge_operations::encode_evm_nft_message(
            [0u8; 20], // destination (zero for stay on ZetaChain)
            receiver_addr,
            asset_identifier,
            &uri,
            sender_addr,
        );
        
        // Call ZetaChain bridge `transfer_and_invoke` via CPI
        let bridge_pda_info = ctx.accounts.bridge_pda.to_account_info();
        let owner_info = ctx.accounts.asset_owner.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();
        
        let receiver_bytes: [u8; 20] = zetachain_universal_contract;

        let call_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.settings.zeta_gateway_program_id, // validated in accounts
            accounts: vec![
                anchor_lang::solana_program::instruction::AccountMeta::new(owner_info.key(), true),        // payer/signer supplying lamports
                anchor_lang::solana_program::instruction::AccountMeta::new(bridge_pda_info.key(), false), // Bridge PDA writable per docs
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(system_program_info.key(), false), // System Program
            ],
            data: bridge_operations::encode_bridge_deposit_and_call_instruction_data(sol_deposit_lamports, receiver_bytes, &cross_chain_message),
        };

        anchor_lang::solana_program::program::invoke(
            &call_ix,
            &[
                owner_info,
                bridge_pda_info,
                system_program_info,
            ],
        )?;
        
        let settings = &mut ctx.accounts.settings;
        settings.message_sequence = settings.message_sequence.checked_add(1).ok_or(Errors::MathOverflow)?;
        
        msg!(
            "Asset bridged to ZetaChain universal contract: {:?}\nDigital asset ID: {:?}\nOrigin network: {}\nTarget network: {}\nTransfer initiator: {}\nDestination address: {}\nMetadata location: {}\nTransfer fee: {}\nTransfer time: {}\nTransfer number: {}",
            zetachain_universal_contract,
            asset_identifier,
            SOLANA_NETWORK_ID,
            final_destination_chain,
            ctx.accounts.asset_owner.key(),
            final_recipient,
            asset_tracker.original_uri.clone(),
            0,
            clock.unix_timestamp,
            settings.message_sequence
        );

        msg!(
            "Digital asset destroyed\nToken mint: {}\nDigital asset ID: {:?}\nPrevious owner: {}\nDestruction time: {}\nDestruction purpose: {}",
            ctx.accounts.mint.key(),
            asset_identifier,
            ctx.accounts.asset_owner.key(),
            clock.unix_timestamp,
            "bridge_to_zetachain".to_string()
        );
        
        Ok(())
    }
    
    /// Validates program state and basic parameters
    fn validate_program_state(&self, final_recipient: &str, sol_deposit_lamports: u64) -> Result<()> {
        require!(!self.settings.paused, Errors::ProgramPaused);
        require!(!final_recipient.is_empty(), Errors::InvalidRecipientAddress);
        require!(final_recipient.len() <= MAX_RECIPIENT_ADDRESS_LENGTH, Errors::InvalidRecipientAddress);
        require!(sol_deposit_lamports >= 2_000_000, Errors::OperationNotAllowed);
        Ok(())
    }

    /// Validates bridge program configuration
    fn validate_bridge_configuration(&self) -> Result<()> {
        require!(
            *self.bridge_pda.owner == self.settings.zeta_gateway_program_id,
            Errors::InvalidBridgeProgram
        );
        Ok(())
    }

    /// Validates asset tracker state and consistency
    fn validate_asset_tracker(&self, asset_identifier: [u8; 32]) -> Result<()> {
        require!(self.asset_tracker.nft_id == asset_identifier, Errors::InvalidDataFormat);
        require!(self.asset_tracker.is_on_solana, Errors::AssetNotOnSolana);
        require_keys_eq!(self.asset_tracker.original_mint, self.mint.key(), Errors::InvalidMint);
        Ok(())
    }

    /// Validates token account ownership and state
    fn validate_token_account(&self) -> Result<()> {
        let expected_owner_ata = anchor_spl::associated_token::get_associated_token_address(
            &self.asset_owner.key(),
            &self.mint.key(),
        );
        require_keys_eq!(expected_owner_ata, self.token_account.key(), Errors::InvalidProgram);
        require_keys_eq!(self.token_account.owner, self.asset_owner.key(), Errors::InvalidProgram);
        require_keys_eq!(self.token_account.mint, self.mint.key(), Errors::InvalidMint);
        require!(self.token_account.amount == 1, Errors::InvalidTokenAmount);
        Ok(())
    }

    /// Validates mint properties for burning
    fn validate_mint_for_burning(&self) -> Result<()> {
        require!(self.mint.decimals == 0, Errors::InvalidMint);
        require!(self.mint.supply == 1, Errors::InvalidTokenSupply);
        Ok(())
    }

    /// Validates recipient address format
    fn validate_recipient_address(&self, final_recipient: &str) -> Result<[u8; 20]> {
        let receiver_addr = inter_chain_helpers::parse_hex_address_to_bytes(final_recipient)
            .map_err(|_| Errors::InvalidRecipientAddress)?;
        Ok(receiver_addr)
    }

    
}