use anchor_lang::prelude::*;
pub mod instructions;
pub mod state;
use instructions::*;
use state::*;
declare_id!("r1cJDNXs7wi6DaLFq4yW2mMA4jVghDZwcT18Bi8yCZn");

#[program]
pub mod metaplex_core {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        reward_point: u16,
        freeze_period: u16,
    ) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        initialize::initialize_handler(ctx, reward_point, freeze_period);
        Ok(())
    }
}
