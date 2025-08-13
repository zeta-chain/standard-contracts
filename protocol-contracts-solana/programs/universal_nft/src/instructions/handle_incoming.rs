use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::{
    instruction as mpl_instruction,
    state::{Creator, DataV2},
};

use crate::state::nft_origin::{NftOrigin, CrossChainNftPayload};
use crate::state::gateway::GatewayConfig;
use crate::state::replay::ReplayMarker;
use crate::utils::{derive_master_edition_pda, derive_metadata_pda, derive_nft_origin_pda, derive_replay_marker_pda};

#[derive(Accounts)]
pub struct HandleIncoming<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub recipient: SystemAccount<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer,
        mint::freeze_authority = payer,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    /// CHECK: metadata PDA
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: master edition PDA
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: gateway program config PDA
    pub gateway_config: UncheckedAccount<'info>,
    /// CHECK: replay marker account
    #[account(mut)]
    pub replay_marker: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<HandleIncoming>, payload: Vec<u8>) -> Result<()> {
    let clock = Clock::get()?;

    // Load gateway config from PDA and verify signer program id from payload origin (out-of-band)
    let (cfg_pda, _bump) = Pubkey::find_program_address(&[GatewayConfig::SEED], &crate::ID);
    require_keys_eq!(ctx.accounts.gateway_config.key(), cfg_pda, ErrorCode::UnauthorizedGateway);
    let data = ctx.accounts.gateway_config.try_borrow_data()?;
    let cfg = GatewayConfig::try_from_slice(&data[8..]).map_err(|_| ErrorCode::UnauthorizedGateway)?;

    // Deserialize payload
    let p: CrossChainNftPayload = CrossChainNftPayload::try_from_slice(&payload)
        .map_err(|_| ErrorCode::InvalidPayload)?;

    // Replay protection: derive and ensure empty
    let (replay_pda, bump) = derive_replay_marker_pda(&p.token_id, p.nonce);
    require_keys_eq!(ctx.accounts.replay_marker.key(), replay_pda, ErrorCode::ReplayPdaMismatch);
    if !ctx.accounts.replay_marker.data_is_empty() {
        return Err(ErrorCode::ReplayAttack.into());
    }
    let space = 8 + ReplayMarker::LEN;
    let lamports = Rent::get()?.minimum_balance(space);
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &replay_pda,
            lamports,
            space as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.replay_marker.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[ReplayMarker::SEED, &p.token_id, &p.nonce.to_le_bytes(), &[bump]]],
    )?;

    // Derive metadata PDAs
    let (metadata_pda, _) = derive_metadata_pda(&ctx.accounts.mint.key());
    let (master_edition_pda, _) = derive_master_edition_pda(&ctx.accounts.mint.key());

    let data_v2 = DataV2 {
        name: format!("Universal NFT #{}", hex::encode(&p.token_id[..8])),
        symbol: "UNFT".to_string(),
        uri: p.metadata_uri.clone(),
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator { address: ctx.accounts.payer.key(), verified: true, share: 100 }]),
        collection: None,
        uses: None,
    };

    // Create metadata
    let ix_meta = mpl_instruction::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        metadata_pda,
        ctx.accounts.mint.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.payer.key(),
        data_v2,
        true,
        None,
        None,
        None,
    );
    anchor_lang::solana_program::program::invoke(
        &ix_meta,
        &[
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    // Create master edition
    let ix_edition = mpl_instruction::create_master_edition_v3(
        mpl_token_metadata::ID,
        master_edition_pda,
        ctx.accounts.mint.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.payer.key(),
        metadata_pda,
        Some(0),
    );
    anchor_lang::solana_program::program::invoke(
        &ix_edition,
        &[
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    // Mint 1 token to the recipient
    anchor_spl::token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        1,
    )?;

    // Create/Update nft_origin PDA (create if missing)
    let (nft_origin_pda, bump) = derive_nft_origin_pda(&p.token_id);
    let space = 8 + NftOrigin::LEN;
    let lamports = Rent::get()?.minimum_balance(space);
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &nft_origin_pda,
            lamports,
            space as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            AccountInfo::new_readonly(&nft_origin_pda, false),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[NftOrigin::SEED, &p.token_id, &[bump]]],
    )?;

    // Serialize NftOrigin
    let origin = NftOrigin {
        origin_chain: p.origin_chain_id,
        origin_token_id: p.token_id,
        origin_mint: p.origin_mint,
        metadata_uri: p.metadata_uri,
        created_at: clock.unix_timestamp,
        bump,
    };
    let mut ai = AccountInfo::new(&nft_origin_pda, false, true, &mut [], &crate::ID, false, 0);
    let mut data = ai.try_borrow_mut_data()?;
    origin.try_serialize(&mut &mut data[..])?;

    emit!(CrossChainMintEvent {
        token_id: p.token_id,
        origin_chain: p.origin_chain_id,
        new_mint: ctx.accounts.mint.key(),
        recipient: p.recipient,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct CrossChainMintEvent {
    pub token_id: [u8; 32],
    pub origin_chain: u16,
    pub new_mint: Pubkey,
    pub recipient: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid payload format")] InvalidPayload,
    #[msg("Unauthorized gateway")] UnauthorizedGateway,
    #[msg("Replay attack")] ReplayAttack,
    #[msg("Replay PDA mismatch")] ReplayPdaMismatch,
}
