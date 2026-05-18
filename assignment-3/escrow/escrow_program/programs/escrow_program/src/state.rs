use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct EscrowAccount {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub expected_amount: u64,
    pub bump: u8,
}
