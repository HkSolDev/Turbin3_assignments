use crate::state::{Config, UpdateAuthority}; // Import UpdateAuthority since it's used below
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface}; // Added TokenInterface import

#[derive(Accounts)] // Fixed typo: 'dervie' -> 'derive'
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"byte", signer.key().as_ref()], // Changed 'seed' to 'seeds' (plural)
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = signer,
        space = 8 + UpdateAuthority::INIT_SPACE, // Changed InitSpace to INIT_SPACE (standard constant name)
        seeds = [b"update_authority", signer.key().as_ref()], // Changed 'seed' to 'seeds'
        bump
    )]
    pub update_authority: Account<'info, UpdateAuthority>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = config,
        mint::freeze_authority = signer,
        seeds = [b"reward", signer.key().as_ref()],
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
        let config = &mut self.config; // Access via self.config, not self.accounts.config
        config.authority = self.signer.key(); // Removed & before key() as it returns a Pubkey copy

        config.reward_bump = reward_bump;
        config.bump = config_bump;
        config.rewards_in_basis_points = reward_point;
        config.freeze_period = freeze_period;

        self.update_authority.authority = self.signer.key();

        Ok(())
    }
}
