use crate::errors::ErrorCode;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use mpl_core::accounts::BaseCollectionV1;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [b"config", collection.key().as_ref()], // Changed to match collection-based seeds
        bump
    )]
    pub config: Account<'info, Config>,

    pub collection: Account<'info, BaseCollectionV1>,

    ///CHECK: This is the update authority for the collection, which is used for signing purpose
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()], // Changed to match collection-based seeds
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = config,
        mint::freeze_authority = signer,
        seeds = [b"reward", collection.key().as_ref()], // Consistent seeds
        bump
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_handler(
        &mut self,
        reward_point: u16,
        freeze_period: u16,
        config_bump: u8,
        reward_bump: u8,
    ) -> Result<()> {
        let config = &mut self.config;
        config.authority = self.signer.key();

        config.reward_bump = reward_bump;
        config.bump = config_bump;
        config.rewards_in_basis_points = reward_point;
        config.freeze_period = freeze_period;

        Ok(())
    }
}
