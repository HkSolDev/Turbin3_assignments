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

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub maker: Pubkey,
    pub price: u64,
    pub asset: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub buyer: Pubkey,
    pub amount: u64,
    pub asset: Pubkey,
    pub bump: u8,
}
