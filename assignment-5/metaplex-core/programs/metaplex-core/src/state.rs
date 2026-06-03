use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub rewards_in_basis_points: u16,
    pub freeze_period: u16,
    pub bump: u8,
    pub reward_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UpdateAuthorityConfig {
    pub authority: Pubkey,
}
