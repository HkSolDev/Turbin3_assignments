use anchor_lang::prelude::*;
use mpl_core::{accounts::BaseCollectionV1, instructions::CreateV2CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,

    ///CHECK: This is the update authority for the collection, which is used for signing purpose only we verify that derives form the correct seeds
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump,
    )]
    pub update_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    ///CHECK:  This is the ID fort the MPL Core Program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintAsset<'info> {
    pub fn mint_asset_handler(&mut self, name: String, uri: String, bump: u8) -> Result<()> {
        let collection_key = self.collection.key();
        let seeds: &[&[&[u8]]] = &[&[b"update_authority", collection_key.as_ref(), &[bump]]];

        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info())) // Pass the update authority account
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(seeds)?;

        Ok(())
    }
}
