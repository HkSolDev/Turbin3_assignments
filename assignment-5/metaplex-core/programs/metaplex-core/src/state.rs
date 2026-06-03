use anchor_lang::prelude::*;

#[account] // Changed from #[derive(account)] because #[account] is the correct Anchor macro
#[derive(InitSpace)] // Added to enable calculation of Config::INIT_SPACE
pub struct Config {
    pub authority: Pubkey,
    pub rewards_in_basis_points: u16,
    pub freeze_period: u16,
    pub bump: u8,
    pub reward_bump: u8,
    pub config_bump: u8,
}

#[account] // Added because it was missing but used in initialize.rs
#[derive(InitSpace)]
pub struct UpdateAuthority {
    pub authority: Pubkey,
}
