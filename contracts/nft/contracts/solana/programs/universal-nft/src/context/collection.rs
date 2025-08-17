use anchor_lang::prelude::*;
use crate::state::{UniversalNftConfig, NftCollection};
use crate::util::constants::*;

#[derive(Accounts)]
#[instruction(collection_name: String)]
pub struct CreateCollection<'info> {
    #[account(
        seeds = [UNIVERSAL_NFT_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, UniversalNftConfig>,
    
    #[account(
        init,
        payer = payer,
        space = NftCollection::LEN,
        seeds = [NFT_COLLECTION_SEED, collection_name.as_bytes()],
        bump
    )]
    pub collection: Account<'info, NftCollection>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(collection_name: String)]
pub struct UpdateCollection<'info> {
    #[account(
        mut,
        seeds = [NFT_COLLECTION_SEED, collection_name.as_bytes()],
        bump = collection.bump,
        has_one = authority
    )]
    pub collection: Account<'info, NftCollection>,
    
    pub authority: Signer<'info>,
}
