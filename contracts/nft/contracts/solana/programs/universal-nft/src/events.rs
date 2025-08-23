use anchor_lang::prelude::*;

#[event]
pub struct CollectionInitialized {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub gateway_address: Pubkey,
}

#[event]
pub struct TokenMinted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub name: String,
    pub uri: String,
}

#[event]
pub struct TokenTransfer {
    pub collection: Pubkey,
    pub token_id: u64,
    pub destination_chain: Vec<u8>,
    pub recipient: Vec<u8>,
    pub uri: String,
    pub sender: Pubkey,
}

#[event]
pub struct TokenTransferReceived {
    pub collection: Pubkey,
    pub token_id: u64,
    pub recipient: Pubkey,
    pub uri: String,
    pub original_sender: Vec<u8>,
}

#[event]
pub struct TokenTransferReverted {
    pub collection: Pubkey,
    pub token_id: u64,
    pub sender: Pubkey,
    pub uri: String,
    pub refund_amount: u64,
}

#[event]
pub struct SetUniversal {
    pub collection: Pubkey,
    pub universal_address: Pubkey,
}

#[event]
pub struct SetConnected {
    pub collection: Pubkey,
    pub chain_id: Vec<u8>,
    pub contract_address: Vec<u8>,
}
