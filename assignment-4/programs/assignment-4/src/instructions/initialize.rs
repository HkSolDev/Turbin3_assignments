use crate::state::Amm;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, space = 8 + Amm::INIT_SPACE, payer = signer, seeds = [Amm::SEED_PREFIX, signer.key().as_ref()], bump)]
    pub amm: Account<'info, Amm>,

    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_a,
        associated_token::authority = amm,
        associated_token::token_program = token_program,
    )]
    pub vault_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_b,
        associated_token::authority = amm,
        associated_token::token_program = token_program,
    )]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = amm,
        mint::freeze_authority = amm,
        seeds = [b"lp_mint".as_ref(), signer.key().as_ref()],
        bump
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_mint,
        associated_token::authority = amm,
        associated_token::token_program = token_program,
    )]
    pub lp_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_amm(&mut self, bumps: &InitializeBumps) -> Result<()> {
        let amm = &mut self.amm;
        amm.admin = self.signer.key();
        amm.mint_a = self.mint_a.key();
        amm.mint_b = self.mint_b.key();
        amm.lp_mint = self.lp_mint.key();
        amm.vault_a = self.vault_a.key();
        amm.vault_b = self.vault_b.key();
        amm.fee = 30; // 0.3% fee
        amm.bump = bumps.amm;
        amm.lp_mint_bump = bumps.lp_mint;
        Ok(())
    }
}
