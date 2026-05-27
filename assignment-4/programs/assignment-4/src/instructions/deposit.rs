use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TransferChecked};

use crate::error::ErrorCode;
use crate::state::Amm;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, has_one = vault_a, has_one = vault_b, has_one = lp_mint)]
    pub amm: Box<Account<'info, Amm>>,

    #[account(mut)]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut, constraint = user_token_a.mint == amm.mint_a && user_token_a.owner == signer.key())]
    pub user_token_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, constraint = user_token_b.mint == amm.mint_b && user_token_b.owner == signer.key())]
    pub user_token_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, constraint = lp_mint.mint_authority.contains(&amm.key()) )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = lp_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub user_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
        constraint = vault_a.mint == amm.mint_a && vault_a.owner == amm.key()
    )]
    pub vault_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, constraint = vault_b.mint == amm.mint_b && vault_b.owner == amm.key()
    )]
    pub vault_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount_a: u64, amount_b: u64) -> Result<()> {
        require!(
            amount_a > 0 && amount_b > 0,
            ErrorCode::AmountMustBeGreaterThanZero
        );

        let reserve_a = self.vault_a.amount;
        let reserve_b = self.vault_b.amount;
        let total_liquidity = self.lp_mint.supply;

        let (deposit_amount_a, deposit_amount_b) = if reserve_a == 0 && reserve_b == 0 {
            (amount_a, amount_b)
        } else {
            let required_b = (amount_a as u128)
                .checked_mul(reserve_b as u128)
                .unwrap()
                .checked_div(reserve_a as u128)
                .unwrap() as u64;

            require!(
                required_b <= amount_b,
                ErrorCode::AmountMustBeGreaterThanZero
            );

            (amount_a, required_b)
        };

        // Transfer token A from user to vault
        let cpi_acc_a = TransferChecked {
            from: self.user_token_a.to_account_info(),
            to: self.vault_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let cpi_ctx_a = CpiContext::new(self.token_program.key(), cpi_acc_a);
        token_interface::transfer_checked(cpi_ctx_a, deposit_amount_a, self.mint_a.decimals)?;

        // Transfer token B from user to vault
        let cpi_acc_b = TransferChecked {
            from: self.user_token_b.to_account_info(),
            to: self.vault_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let cpi_ctx_b = CpiContext::new(self.token_program.key(), cpi_acc_b);
        token_interface::transfer_checked(cpi_ctx_b, deposit_amount_b, self.mint_b.decimals)?;

        let lp_amount = if total_liquidity == 0 {
            ((deposit_amount_a as u128)
                .checked_mul(deposit_amount_b as u128)
                .unwrap() as f64)
                .sqrt() as u64
        } else {
            ((deposit_amount_a as u128)
                .checked_mul(total_liquidity as u128)
                .unwrap()
                .checked_div(reserve_a as u128)
                .unwrap()) as u64
        };

        let cpi_mint_acc = MintTo {
            mint: self.lp_mint.to_account_info(),
            to: self.user_lp_token_account.to_account_info(),
            authority: self.amm.to_account_info(),
        };

        let signer_key = self.signer.key();
        let seeds: &[&[&[u8]]] = &[&[Amm::SEED_PREFIX, signer_key.as_ref(), &[self.amm.bump]]];

        let cpi_mint_ctx =
            CpiContext::new_with_signer(self.token_program.key(), cpi_mint_acc, seeds);
        token_interface::mint_to(cpi_mint_ctx, lp_amount)?;

        Ok(())
    }
}
