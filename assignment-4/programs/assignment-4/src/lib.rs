pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8VwFfb5WFfgaJVUSPtUC7W2QKYWUwBQyk7VBiYNVBv1E");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize_amm(&ctx.bumps)
    }
    pub fn deposit(ctx: Context<Deposit>, amount_a: u64, amount_b: u64) -> Result<()> {
        // FIXED MISTAKE: Previously this called 'deposit(ctx, ...)' which was a recursive call
        // to this same function, causing a stack overflow or infinite loop.
        // It must call the implementation on the accounts struct.
        ctx.accounts.deposit(amount_a, amount_b)
    }

    pub fn withdraw(ctx: Context<WithDraw>, lp_amount: u64) -> Result<()> {
        ctx.accounts.withdraw(lp_amount)
    }
}
