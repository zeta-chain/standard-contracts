use anchor_lang::prelude::*;
use crate::state::Collection;
use crate::SetUniversal as SetUniversalEvent;

#[derive(Accounts)]
pub struct SetUniversalContext<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"collection", collection.authority.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,
    
    pub authority: Signer<'info>,
}

pub fn set_universal(
    ctx: Context<SetUniversalContext>,
    universal_address: Pubkey,
) -> Result<()> {
    let collection = &mut ctx.accounts.collection;
    
    // Validate that the caller is the collection authority
    collection.validate_authority(&ctx.accounts.authority.key())?;
    
    // Update the collection's universal_address field
    collection.universal_address = Some(universal_address);
    
    // Emit a SetUniversal event with collection and universal address
    emit!(SetUniversalEvent {
        collection: collection.key(),
        universal_address,
    });
    
    Ok(())
}