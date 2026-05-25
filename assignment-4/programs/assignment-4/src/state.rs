use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct Amm {
    pub admin: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub lp_mint: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub fee: u16,
    pub bump: u8,
    pub lp_mint_bump: u8,
}
impl Amm {
    pub const SEED_PREFIX: &'static [u8] = b"amm";
}
