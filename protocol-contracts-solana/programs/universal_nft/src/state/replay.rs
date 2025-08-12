use anchor_lang::prelude::*;

#[account]
pub struct ReplayMarker {
    pub bump: u8,
}

impl ReplayMarker {
    pub const SEED: &'static [u8] = b"replay";
}
