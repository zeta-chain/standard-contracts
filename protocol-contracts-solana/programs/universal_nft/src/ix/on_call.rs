use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::gateway::GatewayConfig;
use crate::state::nft_origin::{NftOrigin, CrossChainNftPayload};
use crate::state::replay::ReplayMarker;
use crate::utils::{
    derive_nft_origin_pda,
    derive_replay_marker_pda,
    cpi_create_metadata_v3,
    cpi_create_master_edition_v3,
};

#[derive(Accounts)]
pub struct OnCall<'info> {
    /// The CPI caller program (Gateway) is enforced via address lookup of config
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
    /// CHECK: Metaplex metadata PDA for this mint; created via CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: Metaplex master edition PDA for this mint; created via CPI
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    /// CHECK: nft_origin PDA; will be created with seeds [token_id, "nft_origin"]
    #[account(mut)]
    pub nft_origin: UncheckedAccount<'info>,
    /// CHECK: PDA with gateway program id
    pub gateway_config: UncheckedAccount<'info>,
    /// CHECK: replay marker account
    #[account(mut)]
    pub replay_marker: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// A generic entrypoint to be invoked by ZetaChain Gateway
pub fn handler(ctx: Context<OnCall>, payload: Vec<u8>) -> Result<()> {
    let clock = Clock::get()?;

    // Verify gateway config PDA
    let (cfg_pda, _bump) = Pubkey::find_program_address(&[GatewayConfig::SEED], &crate::ID);
    require_keys_eq!(ctx.accounts.gateway_config.key(), cfg_pda, ErrorCode::UnauthorizedGateway);
    let data = ctx.accounts.gateway_config.try_borrow_data()?;
    let _cfg = GatewayConfig::try_from_slice(&data[8..]).map_err(|_| ErrorCode::UnauthorizedGateway)?;

    // Deserialize payload
    let p: CrossChainNftPayload = CrossChainNftPayload::try_from_slice(&payload)
        .map_err(|_| ErrorCode::InvalidPayload)?;

    // Replay protection: derive and ensure empty
    let (replay_pda, bump) = derive_replay_marker_pda(&p.token_id, p.nonce);
    require_keys_eq!(ctx.accounts.replay_marker.key(), replay_pda, ErrorCode::ReplayPdaMismatch);
    if !ctx.accounts.replay_marker.data_is_empty() {
        return Err(ErrorCode::ReplayAttack.into());
    }
    let space = ReplayMarker::LEN; // includes discriminator
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

    // Write replay marker with discriminator
    let marker = ReplayMarker {
        token_id: p.token_id,
        nonce: p.nonce,
        created_at: clock.unix_timestamp,
        bump,
    };
    let mut data = ctx.accounts.replay_marker.try_borrow_mut_data()?;
    data[..8].copy_from_slice(&ReplayMarker::discriminator());
    marker.try_serialize(&mut &mut data[8..])?;

    // Mint 1 token to recipient
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

    // Create Metaplex metadata and master edition
    cpi_create_metadata_v3(
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.metadata.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        "UniversalNFT".to_string(),
        "UNFT".to_string(),
        p.metadata_uri.clone(),
    )?;

    cpi_create_master_edition_v3(
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.metadata.to_account_info(),
        &ctx.accounts.master_edition.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
    )?;

    // Create nft_origin PDA deterministically
    let (nft_origin_pda, nft_origin_bump) = derive_nft_origin_pda(&p.token_id);
    require_keys_eq!(ctx.accounts.nft_origin.key(), nft_origin_pda, ErrorCode::NftOriginPdaMismatch);

    if ctx.accounts.nft_origin.data_is_empty() {
        let space = NftOrigin::LEN; // includes discriminator
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
                ctx.accounts.nft_origin.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[&p.token_id, b"nft_origin", &[nft_origin_bump]]],
        )?;
    }

    let nft_origin = NftOrigin {
        origin_chain: p.origin_chain_id,
        origin_token_id: p.token_id,
        origin_mint: p.origin_mint,
        metadata_uri: p.metadata_uri,
        created_at: clock.unix_timestamp,
        bump: nft_origin_bump,
    };

    // Write discriminator + data
    let mut no_data = ctx.accounts.nft_origin.try_borrow_mut_data()?;
    no_data[..8].copy_from_slice(&NftOrigin::discriminator());
    nft_origin.try_serialize(&mut &mut no_data[8..])?;

    // Emit cross-chain mint event
    emit!(CrossChainMintEvent {
        token_id: p.token_id,
        origin_chain: p.origin_chain_id,
        recipient: p.recipient,
        nonce: p.nonce,
        timestamp: clock.unix_timestamp,
    });

    msg!("Minted Universal NFT from cross-chain transfer");
    msg!("Token ID: {}", hex::encode(&p.token_id[..8]));
    msg!("Origin Chain: {}", p.origin_chain_id);
    msg!("Recipient: {}", p.recipient);

    Ok(())
}

#[event]
pub struct CrossChainMintEvent {
    pub token_id: [u8; 32],
    pub origin_chain: u16,
    pub recipient: Pubkey,
    pub nonce: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized gateway")]
    UnauthorizedGateway,
    #[msg("Invalid payload")]
    InvalidPayload,
    #[msg("Replay attack detected")]
    ReplayAttack,
    #[msg("Replay PDA mismatch")]
    ReplayPdaMismatch,
    #[msg("Invalid Metadata PDA")]
    InvalidMetadataPda,
    #[msg("Invalid Master Edition PDA")]
    InvalidMasterEditionPda,
    #[msg("Invalid NftOrigin PDA")]
    NftOriginPdaMismatch,
}


