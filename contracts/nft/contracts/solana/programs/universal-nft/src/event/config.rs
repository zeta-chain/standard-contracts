use anchor_lang::prelude::*;

/// Program configuration events
#[event]
pub struct ProgramConfigured {
    /// Program authority
    pub authority: Pubkey,
    /// Gateway program for cross-chain operations
    pub gateway_program: Pubkey,
    /// Timestamp of configuration
    pub timestamp: i64,
}

#[event]
pub struct ProgramConfigUpdated {
    /// Previous authority
    pub old_authority: Option<Pubkey>,
    /// New authority
    pub new_authority: Option<Pubkey>,
    /// Previous gateway program
    pub old_gateway_program: Option<Pubkey>,
    /// New gateway program
    pub new_gateway_program: Option<Pubkey>,
    /// Updated by
    pub updated_by: Pubkey,
    /// Timestamp of update
    pub timestamp: i64,
}
