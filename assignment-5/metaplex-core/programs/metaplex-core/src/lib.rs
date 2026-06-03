use anchor_lang::prelude::*;
pub mod errors;
pub mod instructions;
pub mod state;

use errors::*;
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
        let config_bump = ctx.bumps.config;
        let reward_bump = ctx.bumps.reward_mint;

        msg!("Greetings from: {:?}", ctx.program_id);

        ctx.accounts
            .initialize_handler(reward_point, freeze_period, config_bump, reward_bump)
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        let bump = ctx.bumps.update_authority;
        ctx.accounts.create_collection_handler(name, uri, bump)
    }

    pub fn mint_asset(ctx: Context<MintAsset>, name: String, uri: String) -> Result<()> {
        let bump = ctx.bumps.update_authority;
        ctx.accounts.mint_asset_handler(name, uri, bump)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        stake::handler(ctx)
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        list::handler(ctx, price)
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        delist::handler(ctx)
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        buy::handler(ctx)
    }
}
