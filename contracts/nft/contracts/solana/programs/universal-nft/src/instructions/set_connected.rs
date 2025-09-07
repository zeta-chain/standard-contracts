use anchor_lang::prelude::*;
use crate::state::{Collection, Connected};
use crate::{SetConnected as SetConnectedEvent, UniversalNftError};

/// Set connected contract address for a specific chain
pub fn set_connected(
    ctx: Context<SetConnectedContext>,
    chain_id: Vec<u8>,
    contract_address: Vec<u8>,
) -> Result<()> {
    // Validate that the caller is the collection authority
    let collection = &ctx.accounts.collection;
    collection.validate_authority(&ctx.accounts.authority.key())?;

    // Validate chain_id format (should be 8 bytes for u64)
    require!(
        chain_id.len() == 8,
        UniversalNftError::InvalidDestinationChain
    );

    // Validate contract_address length (max 64 bytes as defined in Connected struct)
    require!(
        contract_address.len() <= 64 && !contract_address.is_empty(),
        UniversalNftError::InvalidRecipientAddress
    );

    // Validate the chain ID is supported
    let chain_id_u64 = u64::from_le_bytes(
        chain_id.as_slice().try_into()
            .map_err(|_| UniversalNftError::InvalidDestinationChain)?
    );
    
    require!(
        crate::is_supported_chain(chain_id_u64),
        UniversalNftError::UnsupportedChain
    );

    // Update the connected account
    let connected = &mut ctx.accounts.connected;
    connected.collection = collection.key();
    connected.chain_id = chain_id.clone();
    connected.contract_address = contract_address.clone();
    connected.bump = ctx.bumps.connected;

    // Emit event
    emit!(SetConnectedEvent {
        collection: collection.key(),
        chain_id: chain_id.clone(),
        contract_address: contract_address.clone(),
    });

    msg!(
        "Connected contract set for collection {} on chain {}: {:?}",
        collection.key(),
        chain_id_u64,
        contract_address
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(chain_id: Vec<u8>)]
pub struct SetConnectedContext<'info> {
    /// Collection account that owns the connected mapping
    #[account(
        has_one = authority,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    /// Authority that can modify the collection settings
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Connected account that stores the chain_id -> contract_address mapping
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Connected::INIT_SPACE,
        seeds = [b"connected", collection.key().as_ref(), chain_id.as_slice()],
        bump
    )]
    pub connected: Account<'info, Connected>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}
