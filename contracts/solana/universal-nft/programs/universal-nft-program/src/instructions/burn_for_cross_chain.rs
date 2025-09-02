use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, Burn, Mint, Token, TokenAccount},
};

use crate::{constants::*, errors::*, state::*, utils::*};

pub fn burn_for_cross_chain(
    ctx: Context<BurnForCrossChain>,
    destination_chain_id: u64,
    destination_address: Vec<u8>,
) -> Result<()> {
    // Extract values from program_config early to avoid borrow conflicts
    let is_initialized;
    let current_nonce;
    {
        let program_config = &ctx.accounts.program_config;
        is_initialized = program_config.is_initialized;
        current_nonce = program_config.nonce;
    }
    
    let nft_state = &mut ctx.accounts.nft_state;
    let clock = Clock::get()?;
    
    require!(
        is_initialized,
        UniversalNftError::ProgramNotInitialized
    );
    
    require!(
        !nft_state.is_cross_chain_locked,
        UniversalNftError::NftLockedForCrossChain
    );
    
    require!(
        destination_chain_id != SOLANA_CHAIN_ID,
        UniversalNftError::InvalidChainId
    );
    
    validate_destination_address(&destination_address)?;
    
    require!(
        ctx.accounts.nft_token_account.amount == NFT_SUPPLY,
        UniversalNftError::InvalidTokenAccount
    );
    
    require!(
        nft_state.cross_chain_history.len() < NftState::MAX_CROSS_CHAIN_HISTORY,
        UniversalNftError::CrossChainHistoryLimitExceeded
    );

    // Burn the NFT
    let burn_cpi_accounts = Burn {
        mint: ctx.accounts.nft_mint.to_account_info(),
        from: ctx.accounts.nft_token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    let burn_cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        burn_cpi_accounts,
    );
    burn(burn_cpi_context, NFT_SUPPLY)?;

    // Record cross-chain transfer
    let cross_chain_transfer = CrossChainTransfer {
        destination_chain_id,
        destination_address: destination_address.clone(),
        transfer_timestamp: clock.unix_timestamp,
        transaction_hash: [0u8; 32], // Will be filled by gateway
        transfer_type: TransferType::Outbound,
    };
    
    nft_state.cross_chain_history.push(cross_chain_transfer);
    nft_state.is_cross_chain_locked = true;

    // Update program statistics
    let program_config = &mut ctx.accounts.program_config;
    program_config.total_cross_chain_transfers = safe_add_u64(
        program_config.total_cross_chain_transfers,
        1
    )?;
    let new_nonce = safe_add_u64(current_nonce, 1)?;
    program_config.nonce = new_nonce;

    // Note: cross_chain_metadata would be used for gateway integration

    // Create cross-chain message for gateway
    let message_type = CrossChainMessageType::BurnConfirmation {
        token_id: nft_state.token_id,
        burned_amount: NFT_SUPPLY,
    };
    
    let message_hash = create_cross_chain_message_hash(
        destination_chain_id,
        new_nonce,
        &message_type,
    )?;

    // Serialize the cross-chain message
    let cross_chain_message = {
        let mut data = Vec::new();
        data.extend_from_slice(&nft_state.token_id.to_le_bytes());
        data.extend_from_slice(&destination_chain_id.to_le_bytes());
        data.extend_from_slice(&(destination_address.len() as u32).to_le_bytes());
        data.extend_from_slice(&destination_address);
        data.extend_from_slice(&message_hash);
        data
    };

    // Call gateway program for cross-chain transfer
    crate::instructions::gateway_handlers::call_gateway_deposit_and_call(
        ctx.accounts.gateway_program.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        0, // No SOL transfer for NFT burn
        destination_address.clone(),
        cross_chain_message,
    ).map_err(|_| UniversalNftError::GatewayCallFailed)?;

    msg!("NFT burned for cross-chain transfer");
    msg!("Token ID: {}", nft_state.token_id);
    msg!("Destination Chain: {}", destination_chain_id);
    msg!("Destination Address: {:?}", destination_address);
    msg!("Message Hash: {:?}", message_hash);

    Ok(())
}

#[derive(Accounts)]
pub struct BurnForCrossChain<'info> {
    #[account(
        mut,
        seeds = [PROGRAM_SEED],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        seeds = [NFT_STATE_SEED, nft_mint.key().as_ref()],
        bump = nft_state.bump,
        constraint = nft_state.mint == nft_mint.key() @ UniversalNftError::InvalidTokenAccount,
    )]
    pub nft_state: Account<'info, NftState>,

    #[account(
        mut,
        mint::authority = owner,
        mint::freeze_authority = owner,
        constraint = nft_mint.supply == NFT_SUPPLY @ UniversalNftError::InvalidTokenAccount,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = owner,
        constraint = nft_token_account.amount == NFT_SUPPLY @ UniversalNftError::InvalidTokenAccount,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner.key() == nft_state.original_owner @ UniversalNftError::InvalidAuthority,
    )]
    pub owner: Signer<'info>,

    /// CHECK: ZetaChain gateway program for cross-chain operations
    #[account(
        constraint = gateway_program.key() == program_config.gateway_program_id @ UniversalNftError::InvalidGatewayProgramId,
    )]
    pub gateway_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}