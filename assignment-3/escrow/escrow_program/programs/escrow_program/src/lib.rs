pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Ebvs4JtK8PS82ZR981QRTvRLdn3fX5iWBm9sRJ3NUjBv");


#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        instructions::make::make_handler(ctx, seed, receive, amount)
    }
    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::take_handler(ctx)
    }
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::refund_handler(ctx)
    }
}