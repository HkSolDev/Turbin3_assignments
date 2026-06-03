use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::TransferV1CpiBuilder,
};
use crate::state::Listing;

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: The maker who will receive the funds
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = asset.owner == listing.key(),
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(
        mut,
        close = maker,
        seeds = [b"listing", asset.key().as_ref()],
        bump = listing.bump,
        constraint = listing.maker == maker.key(),
    )]
    pub listing: Account<'info, Listing>,

    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Buy>) -> Result<()> {
    let asset_key = ctx.accounts.asset.key();
    let price = ctx.accounts.listing.price;
    let listing_bump = ctx.accounts.listing.bump;
    let seeds: &[&[&[u8]]] = &[&[b"listing", asset_key.as_ref(), &[listing_bump]]];

    // 1. Transfer SOL from taker to maker
    anchor_lang::system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.taker.to_account_info(),
                to: ctx.accounts.maker.to_account_info(),
            },
        ),
        price,
    )?;

    // 2. Transfer the asset from the listing PDA to the taker
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .new_owner(&ctx.accounts.taker.to_account_info())
        .payer(&ctx.accounts.taker.to_account_info())
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke_signed(seeds)?;

    msg!("Asset {} purchased for {} lamports", asset_key, price);

    Ok(())
}
