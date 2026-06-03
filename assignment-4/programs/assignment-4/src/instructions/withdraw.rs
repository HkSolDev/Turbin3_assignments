use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    self, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::state::Amm;

#[derive(Accounts)]
pub struct WithDraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // FIXED MISTAKE: Added Box<> here as well to prevent stack overflow.
    #[account(
        mut,
        has_one = vault_a,
        has_one = vault_b,
        has_one = lp_mint,
        has_one = mint_a,
        has_one = mint_b,
    )]
    pub amm: Box<Account<'info, Amm>>,

    #[account(mut)]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub vault_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub vault_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, constraint = lp_mint.mint_authority.contains(&amm.key()))]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        constraint = user_token_a.mint == amm.mint_a && user_token_a.owner == signer.key()
    )]
    pub user_token_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_token_b.mint == amm.mint_b && user_token_b.owner == signer.key()
    )]
    pub user_token_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_lp_token_account.mint == amm.lp_mint && user_lp_token_account.owner == signer.key()
    )]
    pub user_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> WithDraw<'info> {
    pub fn withdraw(&mut self, lp_amount: u64) -> Result<()> {
        let amount_a = (lp_amount as u128)
            .checked_mul(self.vault_a.amount as u128)
            .unwrap()
            .checked_div(self.lp_mint.supply as u128)
            .unwrap() as u64;

        let amount_b = (lp_amount as u128)
            .checked_mul(self.vault_b.amount as u128)
            .unwrap()
            .checked_div(self.lp_mint.supply as u128)
            .unwrap() as u64;

        let admin_key = self.amm.admin.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[Amm::SEED_PREFIX, admin_key.as_ref(), &[self.amm.bump]]];

        // Transfer Token A to user
        let cpi_acc_a = TransferChecked {
            from: self.vault_a.to_account_info(),
            to: self.user_token_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.amm.to_account_info(),
        };

        // FIXED MISTAKE: CpiContext expects the Program ID (Pubkey), not AccountInfo.
        // Use self.token_program.key() instead of .to_account_info().
        let cpi_ctx_a =
            CpiContext::new_with_signer(self.token_program.key(), cpi_acc_a, signer_seeds);
        token_interface::transfer_checked(cpi_ctx_a, amount_a, self.mint_a.decimals)?;

        // FIXED MISTAKE: Previously, Token B transfer was using Token A's accounts
        // (vault_a, user_token_a, mint_a). Corrected to use Token B assets.
        let cpi_acc_b = TransferChecked {
            from: self.vault_b.to_account_info(),
            to: self.user_token_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.amm.to_account_info(),
        };

        let cpi_ctx_b =
            CpiContext::new_with_signer(self.token_program.key(), cpi_acc_b, signer_seeds);
        token_interface::transfer_checked(cpi_ctx_b, amount_b, self.mint_b.decimals)?;

        // Burn LP Tokens from user
        let cpi_burn_acc = Burn {
            mint: self.lp_mint.to_account_info(),
            from: self.user_lp_token_account.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let cpi_ctx_burn = CpiContext::new(self.token_program.key(), cpi_burn_acc);
        token_interface::burn(cpi_ctx_burn, lp_amount)?;

        Ok(())
    }
}
