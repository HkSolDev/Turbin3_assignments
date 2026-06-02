use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
#[dervie(Account)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init,
    payer= signer,
    space = 8 + Config::InitSpace,
    seed = [b"byte", signer.key().as_ref()],
    bump
        )]
    pub config: Account<'info, Config>,

    #[account(init,
payer = signer,
space = 8 + UpdateAuthority::InitSpace,
seed = [b"update_authority", signer.key().as_ref()],
bump
)]
    pub update_authority: Account<'info, UpdateAuthority>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = config,
        mint::freeze_authority = signer,
        mint::seed = [b"reward", signer.key().as_ref()],
        bump
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_handler(
        ctx: Context<Initialize>,
        reward_point: u16,
        freeze_period: u16,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.signer.key();
        config.bump = ctx.bumps.config;
        config.reward_bump = ctx.bumps.reward_mint;
        config.rewards_in_basis_points = reward_point;
        config.freeze_period = freeze_period;
        Ok(())
    }
}
