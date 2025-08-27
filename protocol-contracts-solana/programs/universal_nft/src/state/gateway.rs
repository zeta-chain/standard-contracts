use anchor_lang::prelude::*;

#[account]
pub struct GatewayConfig {
    pub gateway_program: Pubkey,
    pub gateway_pda: Pubkey,
    pub bump: u8,
}

impl GatewayConfig {
    pub const SEED: &'static [u8] = b"gateway_config";
    // discriminator (8) + gateway_program(32) + gateway_pda(32) + bump(1)
    pub const LEN: usize = 8 + 32 + 32 + 1;
}
