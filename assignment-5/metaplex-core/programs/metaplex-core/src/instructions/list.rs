use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::TransferV1CpiBuilder,
};
use crate::state::Listing;

#[derive(Accounts)]
pub struct List<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mut,
        constraint = asset.owner == maker.key(),
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(
        init,
        payer = maker,
        space = 8 + Listing::INIT_SPACE,
        seeds = [b"listing", asset.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,

    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<List>, price: u64) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    listing.maker = ctx.accounts.maker.key();
    listing.price = price;
    listing.asset = ctx.accounts.asset.key();
    listing.bump = ctx.bumps.listing;

    // Transfer the asset to the listing PDA (escrow)
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .new_owner(&ctx.accounts.listing.to_account_info())
        .payer(&ctx.accounts.maker.to_account_info())
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke()?;

    msg!("Asset {} listed for {} lamports", ctx.accounts.asset.key(), price);

    Ok(())
}
