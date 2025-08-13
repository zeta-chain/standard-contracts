use anchor_lang::prelude::*;

#[account]
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GatewayConfig {
    pub gateway_program: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
}

impl GatewayConfig {
    pub const SEED: &'static [u8] = b"gateway_config";
    pub const LEN: usize = 32 + 32 + 1;
}


