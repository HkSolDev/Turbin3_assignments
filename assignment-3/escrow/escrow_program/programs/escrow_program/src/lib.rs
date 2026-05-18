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
pub mod escrow_program {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        amount_deposite: u64,
        receive_amount: u64,
    ) -> Result<()> {
        initialize::handler(ctx, amount_deposite, receive_amount)
    }

    pub fn take(ctx: Context<Taker>) -> Result<()> {
        take::handler(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        refund::handler(ctx)
    }
}
