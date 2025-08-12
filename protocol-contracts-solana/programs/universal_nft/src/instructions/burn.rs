use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use sha2::{Digest, Sha256};

use crate::state::nft_origin::NftOrigin;
use crate::state::replay::ReplayMarker;
use crate::utils::*;

#[derive(Accounts)]
pub struct BurnForTransfer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub owner_token_account: UncheckedAccount<'info>,
    /// CHECK: nft_origin PDA to read origin information
    pub nft_origin: UncheckedAccount<'info>,
    #[account(mut)]
    pub replay_marker: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BurnForTransfer>, nonce: u64) -> Result<()> {
    let clock = Clock::get()?;
    
    // Verify the owner owns the token
    let token_account = anchor_spl::token::TokenAccount::try_from(&ctx.accounts.owner_token_account)?;
    if token_account.owner != ctx.accounts.owner.key() {
        return Err(ErrorCode::UnauthorizedOwner.into());
    }
    
    if token_account.amount == 0 {
        return Err(ErrorCode::NoTokensToBurn.into());
    }
    
    // Read the nft_origin PDA to get token information
    let nft_origin_data = NftOrigin::try_from(&ctx.accounts.nft_origin)?;
    
    // Check replay protection
    let (replay_marker_pda, bump) = derive_replay_marker_pda(&nft_origin_data.origin_token_id, nonce);
    let replay_marker = &mut ctx.accounts.replay_marker;
    
    // Initialize replay marker if it doesn't exist
    if replay_marker.data_is_empty() {
        let replay_data = ReplayMarker {
            token_id: nft_origin_data.origin_token_id.clone(),
            nonce,
            created_at: clock.unix_timestamp,
            bump,
        };
        replay_marker.assign(&replay_marker_pda);
        replay_marker.set_inner(replay_data);
    } else {
        return Err(ErrorCode::ReplayAttempt.into());
    }
    
    // Burn the token
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
    
    // Prepare cross-chain payload
    let payload = CrossChainNftPayload {
        version: 1,
        token_id: nft_origin_data.origin_token_id,
        origin_chain_id: nft_origin_data.origin_chain,
        origin_mint: nft_origin_data.origin_mint,
        metadata_uri: nft_origin_data.metadata_uri,
        recipient: ctx.accounts.owner.key(),
        nonce,
    };
    
    // Emit cross-chain transfer event
    emit!(CrossChainTransferEvent {
        token_id: payload.token_id.clone(),
        origin_chain: payload.origin_chain_id,
        destination_chain: 0, // Will be set by gateway
        owner: payload.recipient,
        nonce: payload.nonce,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("🔥 NFT burned successfully for cross-chain transfer!");
    msg!("Token ID: {}", hex::encode(&payload.token_id));
    msg!("Origin Chain: {}", payload.origin_chain_id);
    msg!("Nonce: {}", nonce);
    
    Ok(())
}

#[event]
pub struct CrossChainTransferEvent {
    pub token_id: Vec<u8>,
    pub origin_chain: u16,
    pub destination_chain: u16,
    pub owner: Pubkey,
    pub nonce: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized owner")]
    UnauthorizedOwner,
    #[msg("No tokens to burn")]
    NoTokensToBurn,
    #[msg("Replay attempt detected")]
    ReplayAttempt,
    #[msg("Invalid NFT origin account")]
    InvalidNftOriginAccount,
}

// Cross-chain payload structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrossChainNftPayload {
    pub version: u8,
    pub token_id: Vec<u8>,
    pub origin_chain_id: u16,
    pub origin_mint: Pubkey,
    pub metadata_uri: String,
    pub recipient: Pubkey,
    pub nonce: u64,
}
