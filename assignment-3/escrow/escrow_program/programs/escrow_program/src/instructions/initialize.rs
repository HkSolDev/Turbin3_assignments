use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::EscrowAccount;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
init,
payer = maker,
space = EscrowAccount::INIT_SPACE,
seeds = [b"escrow", maker.key().as_ref() ],
bump
        )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_account,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<Initialize>, amount_deposite: u64, receive_ammount: u64) -> Result<()> {
    msg!(" Initialize the escrow{:?}", ctx.program_id);

    let escrow_account = &mut ctx.accounts.escrow_account;

    escrow_account.maker = ctx.accounts.maker.key();
    escrow_account.mint_a = ctx.accounts.mint_a.key();
    escrow_account.mint_b = ctx.accounts.mint_b.key();
    escrow_account.expected_amount = receive_ammount;
    escrow_account.bump = ctx.bumps.escrow_account;

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.maker_ata_a.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        mint: ctx.accounts.mint_a.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
    token_interface::transfer_checked(cpi_ctx, amount_deposite, ctx.accounts.mint_a.decimals)?;

    Ok(())
}
