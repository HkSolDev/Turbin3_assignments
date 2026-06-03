use anchor_lang::prelude::*;
pub mod instructions;
pub mod state;
use instructions::*;

declare_id!("r1cJDNXs7wi6DaLFq4yW2mMA4jVghDZwcT18Bi8yCZn");

#[program]
pub mod metaplex_core {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        reward_point: u16,
        freeze_period: u16,
    ) -> Result<()> {
        // Use 'let' to declare variables and access bumps directly from the ctx.bumps struct
        let config_bump = ctx.bumps.config;
        let reward_bump = ctx.bumps.reward_mint;

        msg!("Greetings from: {:?}", ctx.program_id);

        // Call the handler on ctx.accounts since it's an 'impl' on the struct
        ctx.accounts
            .initialize_handler(reward_point, freeze_period, config_bump, reward_bump)
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts
            .create_collection_handler(name, uri, ctx.bumps.update_authority)
    }

    pub fn mint_asset(ctx: Context<MintAsset>, name: String, uri: String) -> Result<()> {
        ctx.accounts
            .mint_asset_handler(name, uri, ctx.bumps.update_authority)
    }
}
