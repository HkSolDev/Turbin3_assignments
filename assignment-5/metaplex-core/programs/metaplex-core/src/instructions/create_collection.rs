use crate::state::UpdateAuthorityConfig;
use anchor_lang::prelude::*;
use mpl_core::{instructions::CreateCollectionV2CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub collection: Signer<'info>,

    #[account(
        mut,
        seeds = [b"update_authority", collection.key().as_ref()],
        bump,
    )]
    pub update_authority: Account<'info, UpdateAuthorityConfig>, // Changed from UncheckedAccount<'info, UpdateAuthorityConfig> to Account<'info, UpdateAuthorityConfig>

    pub system_program: Program<'info, System>,

    /// CHECK: This is the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>, // Changed from Program<'info, MPLCoreId> to UncheckedAccount for simpler CPI
}

impl<'info> CreateCollection<'info> {
    pub fn create_collection_handler(&mut self, name: String, uri: String, bump: u8) -> Result<()> {
        let collection_key = self.collection.key(); // Get the collection key once and reuse it
        let seeds: &[&[&[u8]]] = &[&[b"update_authority", collection_key.as_ref(), &[bump]]];

        CreateCollectionV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.signer.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info())) // Pass the update authority account
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(seeds)?;

        Ok(())
    }
}
