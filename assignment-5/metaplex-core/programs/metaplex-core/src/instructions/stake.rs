use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{AddPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{Attribute, Attributes, Plugin, PluginAuthority, PluginType},
};

use crate::errors::ErrorCode;
use crate::state::Config;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = asset.owner == signer.key() @ ErrorCode::InvalidOwner,
        constraint = asset.update_authority == mpl_core::types::UpdateAuthority::Collection(collection.key()) @ ErrorCode::InvalidUpdateAuthority
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is the update authority for the collection, used for signing CPIs
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority_pda: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Stake>) -> Result<()> {
    // 1. Fetch existing attributes (if any)
    let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .ok()
    .map(|(_, attrs, _)| attrs);

    // 2. Prepare the Attributes list
    let mut attributes_list: Vec<Attribute> = Vec::new();
    let mut has_attributes_plugin = false;

    if let Some(attributes) = &attributes_fetched {
        has_attributes_plugin = true;
        for attribute in &attributes.attribute_list {
            if attribute.key == "staked" {
                require!(attribute.value == "false", ErrorCode::AlreadyStaked);
            } else if attribute.key != "staked_at" {
                attributes_list.push(attribute.clone());
            }
        }
    }

    // 3. Add the staking attributes
    attributes_list.push(Attribute {
        key: "staked".to_string(),
        value: "true".to_string(),
    });
    attributes_list.push(Attribute {
        key: "staked_at".to_string(),
        value: Clock::get()?.unix_timestamp.to_string(),
    });

    let collection_key = ctx.accounts.collection.key();
    let update_authority_bump = ctx.bumps.update_authority_pda;
    let seeds: &[&[&[u8]]] = &[&[
        b"update_authority",
        collection_key.as_ref(),
        &[update_authority_bump],
    ]];

    // 4. Update or Add the plugin
    if has_attributes_plugin {
        UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.asset.to_account_info())
            .collection(Some(&ctx.accounts.collection.to_account_info()))
            .payer(&ctx.accounts.signer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: attributes_list,
            }))
            .invoke_signed(seeds)?;
    } else {
        AddPluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.asset.to_account_info())
            .collection(Some(&ctx.accounts.collection.to_account_info()))
            .payer(&ctx.accounts.signer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: attributes_list,
            }))
            .authority(Some(&ctx.accounts.update_authority_pda.to_account_info()))
            .invoke_signed(seeds)?;
    }

    msg!("Asset {} staked successfully", ctx.accounts.asset.key());

    Ok(())
}
