use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::TransferV1CpiBuilder,
};
use crate::state::Listing;

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

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

pub fn handler(ctx: Context<Delist>) -> Result<()> {
    let asset_key = ctx.accounts.asset.key();
    let listing_bump = ctx.accounts.listing.bump;
    let seeds: &[&[&[u8]]] = &[&[b"listing", asset_key.as_ref(), &[listing_bump]]];

    // Transfer the asset back to the maker
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .new_owner(&ctx.accounts.maker.to_account_info())
        .payer(&ctx.accounts.maker.to_account_info())
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke_signed(seeds)?;

    msg!("Asset {} delisted", asset_key);

    Ok(())
}
