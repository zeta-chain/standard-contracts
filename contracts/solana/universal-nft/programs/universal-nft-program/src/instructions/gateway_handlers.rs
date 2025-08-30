use anchor_lang::prelude::*;
use borsh::BorshDeserialize;

use crate::{errors::*, state::*, utils::*};

pub fn on_call(ctx: Context<OnCall>, sender: [u8; 20], message: Vec<u8>) -> Result<()> {
    // Extract values from program_config early to avoid borrow conflicts
    let is_initialized;
    let gateway_program_id;
    let current_nonce;
    {
        let program_config = &ctx.accounts.program_config;
        is_initialized = program_config.is_initialized;
        gateway_program_id = program_config.gateway_program_id;
        current_nonce = program_config.nonce;
    }
    
    let clock = Clock::get()?;
    
    require!(
        is_initialized,
        UniversalNftError::ProgramNotInitialized
    );
    
    // Verify call comes from gateway program
    let instruction_sysvar = &ctx.accounts.instruction_sysvar;
    let current_instruction = solana_program::sysvar::instructions::get_instruction_relative(0, instruction_sysvar)?;
    
    require!(
        current_instruction.program_id == gateway_program_id,
        UniversalNftError::UnauthorizedCrossChainOperation
    );
    
    // Parse the cross-chain message
    let parsed_message: CrossChainMessageType = match CrossChainMessageType::try_from_slice(&message) {
        Ok(msg) => msg,
        Err(_) => return Err(UniversalNftError::InvalidMessageFormat.into()),
    };
    
    match parsed_message {
        CrossChainMessageType::MintRequest { recipient, metadata } => {
            msg!("Received mint request from chain");
            msg!("Sender: {:?}", sender);
            msg!("Recipient: {}", recipient);
            msg!("Metadata: {:?}", metadata);
            
            // The actual minting will be handled by mint_from_cross_chain instruction
            // This is just for logging and validation
        },
        CrossChainMessageType::BurnConfirmation { token_id, burned_amount } => {
            msg!("Received burn confirmation");
            msg!("Token ID: {}", token_id);
            msg!("Burned Amount: {}", burned_amount);
        },
        CrossChainMessageType::RevertRequest { original_transaction, revert_context } => {
            msg!("Received revert request");
            msg!("Original Transaction: {:?}", original_transaction);
            msg!("Revert Context: {:?}", revert_context);
        },
    }
    
    // Update nonce to prevent replay
    let program_config = &mut ctx.accounts.program_config;
    program_config.nonce = safe_add_u64(current_nonce, 1)?;
    
    Ok(())
}

pub fn on_revert(ctx: Context<OnRevert>, revert_context: RevertContext) -> Result<()> {
    // Extract values first to avoid borrow conflicts
    let program_config_bump;
    let current_nonce;
    let gateway_program_id;
    let is_initialized;
    
    {
        let program_config = &ctx.accounts.program_config;
        program_config_bump = program_config.bump;
        current_nonce = program_config.nonce;
        gateway_program_id = program_config.gateway_program_id;
        is_initialized = program_config.is_initialized;
    }
    
    require!(is_initialized, UniversalNftError::ProgramNotInitialized);
    
    // Verify call comes from gateway program
    let instruction_sysvar = &ctx.accounts.instruction_sysvar;
    let current_instruction = solana_program::sysvar::instructions::get_instruction_relative(0, instruction_sysvar)?;
    
    require!(
        current_instruction.program_id == gateway_program_id,
        UniversalNftError::UnauthorizedCrossChainOperation
    );
    
    let nft_state = &mut ctx.accounts.nft_state;
    let clock = Clock::get()?;
    
    // Verify the token ID matches the revert request
    let token_id_from_bytes = u64::from_le_bytes(
        revert_context.token_id.as_slice().try_into()
            .map_err(|_| UniversalNftError::InvalidTokenId)?
    );
    
    require!(
        nft_state.token_id == token_id_from_bytes,
        UniversalNftError::InvalidTokenId
    );
    
    require!(
        nft_state.is_cross_chain_locked,
        UniversalNftError::InvalidRevertContext
    );
    
    msg!("Executing revert operation for failed cross-chain transfer");
    msg!("Token ID: {}", nft_state.token_id);
    msg!("Original Chain: {}", revert_context.original_chain_id);
    msg!("Revert Reason: {}", revert_context.revert_message);
    
    // REVERT LOGIC: Re-mint the NFT that was burned for cross-chain transfer
    let mint_to_cpi_accounts = anchor_spl::token::MintTo {
        mint: ctx.accounts.nft_mint.to_account_info(),
        to: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.program_config.to_account_info(),
    };
    
    let seeds = &[crate::state::PROGRAM_SEED, &[program_config_bump]];
    let signer = &[&seeds[..]];
    
    let mint_to_cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        mint_to_cpi_accounts,
        signer,
    );
    
    anchor_spl::token::mint_to(mint_to_cpi_context, crate::constants::NFT_SUPPLY)?;
    
    // Update state
    nft_state.is_cross_chain_locked = false;
    
    // Record the revert in cross-chain history
    if let Some(last_transfer) = nft_state.cross_chain_history.last_mut() {
        last_transfer.transaction_hash = revert_context.revert_message.as_bytes()[..32].try_into().unwrap_or([0u8; 32]);
    }
    
    if nft_state.cross_chain_history.len() < crate::state::NftState::MAX_CROSS_CHAIN_HISTORY {
        let revert_record = crate::state::CrossChainTransfer {
            destination_chain_id: crate::state::SOLANA_CHAIN_ID,
            destination_address: nft_state.original_owner.to_bytes().to_vec(),
            transfer_timestamp: clock.unix_timestamp,
            transaction_hash: [0u8; 32],
            transfer_type: crate::state::TransferType::Inbound,
        };
        nft_state.cross_chain_history.push(revert_record);
    }
    
    // Update program statistics
    let program_config = &mut ctx.accounts.program_config;
    program_config.nonce = safe_add_u64(current_nonce, 1)?;
    
    msg!("Revert completed successfully");
    msg!("NFT re-minted to original owner: {}", nft_state.original_owner);
    
    Ok(())
}

#[derive(Accounts)]
pub struct OnCall<'info> {
    #[account(
        mut,
        seeds = [PROGRAM_SEED],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,
    
    /// CHECK: This account is safe because it's only used to read instruction data
    #[account(address = solana_program::sysvar::instructions::id())]
    pub instruction_sysvar: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct OnRevert<'info> {
    #[account(
        mut,
        seeds = [crate::state::PROGRAM_SEED],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,
    
    #[account(
        mut,
        seeds = [crate::state::NFT_STATE_SEED, nft_mint.key().as_ref()],
        bump = nft_state.bump,
    )]
    pub nft_state: Account<'info, NftState>,
    
    #[account(
        mut,
        mint::authority = program_config,
        mint::freeze_authority = program_config,
    )]
    pub nft_mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_state.original_owner,
    )]
    pub owner_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    pub token_program: Program<'info, anchor_spl::token::Token>,
    
    /// CHECK: This account is safe because it's only used to read instruction data
    #[account(address = solana_program::sysvar::instructions::id())]
    pub instruction_sysvar: UncheckedAccount<'info>,
}

// Helper function to call gateway program for cross-chain operations
pub fn call_gateway_deposit_and_call<'a>(
    gateway_program: AccountInfo<'a>,
    payer: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    amount: u64,
    receiver: Vec<u8>,
    message: Vec<u8>,
) -> Result<()> {
    // Construct the CPI instruction for gateway deposit_and_call
    let deposit_and_call_ix = solana_program::instruction::Instruction {
        program_id: *gateway_program.key,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*payer.key, true),
            solana_program::instruction::AccountMeta::new_readonly(*system_program.key, false),
        ],
        data: {
            let mut data = vec![0]; // Instruction discriminator placeholder
            data.extend_from_slice(&amount.to_le_bytes());
            data.extend_from_slice(&(receiver.len() as u32).to_le_bytes());
            data.extend_from_slice(&receiver);
            data.extend_from_slice(&(message.len() as u32).to_le_bytes());
            data.extend_from_slice(&message);
            data
        },
    };

    // Execute the CPI
    solana_program::program::invoke(
        &deposit_and_call_ix,
        &[gateway_program, payer, system_program],
    )?;

    msg!("Successfully called gateway deposit_and_call");
    msg!("Amount: {}", amount);
    msg!("Receiver: {:?}", receiver);
    msg!("Message length: {}", message.len());
    
    Ok(())
}

// Function to call gateway for asset withdrawal
pub fn call_gateway_withdraw<'a>(
    gateway_program: AccountInfo<'a>,
    receiver: Pubkey,
    amount: u64,
    asset: Pubkey,
) -> Result<()> {
    let withdraw_ix = solana_program::instruction::Instruction {
        program_id: *gateway_program.key,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(receiver, false),
            solana_program::instruction::AccountMeta::new(asset, false),
        ],
        data: {
            let mut data = vec![1]; // Withdraw instruction discriminator
            data.extend_from_slice(&amount.to_le_bytes());
            data
        },
    };

    solana_program::program::invoke_signed(
        &withdraw_ix,
        &[gateway_program],
        &[], // No seeds needed for withdrawal
    )?;

    msg!("Successfully called gateway withdraw");
    Ok(())
}