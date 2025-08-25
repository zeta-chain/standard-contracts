use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Collection {
    pub authority: Pubkey,
    #[max_len(32)]
    pub name: String,
    #[max_len(10)]
    pub symbol: String,
    #[max_len(200)]
    pub uri: String,
    pub gateway_address: Pubkey,
    pub universal_address: Option<Pubkey>,
    pub next_token_id: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Connected {
    pub collection: Pubkey,
    #[max_len(32)]
    pub chain_id: Vec<u8>,
    #[max_len(64)]
    pub contract_address: Vec<u8>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrossChainMessage {
    pub destination_chain: Vec<u8>,
    pub recipient: Vec<u8>,
    pub token_id: u64,
    pub uri: String,
    pub sender: Vec<u8>,
}
