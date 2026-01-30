use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1}, 
    fetch_plugin, 
    instructions::{AddPluginV1CpiBuilder, UpdatePluginV1CpiBuilder}, 
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority}
};
use crate::state::Config;
use crate::errors::StakingError;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub update_authority: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    /// CHECK: This is the NFT account
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: This is the collection account
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> Stake<'info> {
    pub fn stake(&mut self) -> Result<()> {
        
        // TBD: Perform this validations in account constraints (currently BaseAssetV1 and BaseCollectionV1 are given errors)
        // Verify NFT owner and update authority
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(base_asset.owner == self.user.key(), StakingError::InvalidOwner);
        require!(base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()), StakingError::InvalidAuthority);
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(base_collection.update_authority == self.update_authority.key(), StakingError::InvalidAuthority);

        // Check if the NFT has the attribute plugin already added
        match fetch_plugin::<BaseAssetV1, Attributes>(&self.nft.to_account_info(), PluginType::Attributes) {
            Err(_) => {
                // Add the attribute plugin to the NFT if it doesn't have it yet ('staked' and 'staked_at' attributes)
                AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.nft.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(
                        Attributes { 
                            attribute_list: vec![
                                Attribute { 
                                    key: "staked".to_string(), 
                                    value: "true".to_string() 
                                },
                                Attribute { 
                                    key: "staked_at".to_string(), 
                                    value: Clock::get()?.unix_timestamp.to_string() 
                                },
                            ] 
                        }
                    ))
                    .init_authority(PluginAuthority::UpdateAuthority)   // TODO: Check what this does
                    .invoke()?;
            }
            Ok((_, fetched_attribute_list, _)) => {
                // Verify the fetched attribute list has the 'staked' and 'staked_at' attributes
                let mut attribute_list: Vec<Attribute> = Vec::new();
                let mut staked = false;
                let mut staked_at = false;
                for attribute in fetched_attribute_list.attribute_list {
                    if attribute.key == "staked" {
                        require!(attribute.value == "false", StakingError::AlreadyStaked);
                        attribute_list.push(Attribute { 
                            key: "staked".to_string(), 
                            value: "true".to_string() 
                        });
                        staked = true;
                    }else if attribute.key == "staked_at" {
                        attribute_list.push(Attribute { 
                            key: "staked_at".to_string(), 
                            value: Clock::get()?.unix_timestamp.to_string() 
                        });
                        staked_at = true;
                    }else {
                        attribute_list.push(attribute);
                    }
                }
                // Add the 'staked' and 'staked_at' attributes if they don't exist
                if !staked {
                    attribute_list.push(Attribute { 
                        key: "staked".to_string(), 
                        value: "true".to_string() 
                    });
                }
                if !staked_at {
                    attribute_list.push(Attribute { 
                        key: "staked_at".to_string(), 
                        value: Clock::get()?.unix_timestamp.to_string() 
                    });
                }
                UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.nft.to_account_info())
                .collection(Some(&self.collection.to_account_info()))
                .payer(&self.user.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .plugin(Plugin::Attributes( Attributes { attribute_list }))
                .invoke()?;
            }
        }

        // Freeze the NFT
        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
            .init_authority(PluginAuthority::UpdateAuthority)
            .invoke()?;

        Ok(())
    }
}