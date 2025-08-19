use anchor_lang::prelude::*;

#[account]
pub struct GatewayConfig {
    pub gateway_program: Pubkey,
    pub bump: u8,
}

impl GatewayConfig {
    pub const SEED: &'static [u8] = b"gateway_config";
    pub const LEN: usize = 32 + 1;
}


