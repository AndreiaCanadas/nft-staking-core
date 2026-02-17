use anchor_lang::prelude::*;
use anchor_spl::{token_interface::{Mint, TokenInterface, TokenAccount, MintToChecked, mint_to_checked}, associated_token::AssociatedToken};
use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::{BaseAssetV1, BaseCollectionV1}, 
    fetch_plugin, 
    instructions::UpdatePluginV1CpiBuilder,
    types::{Attribute, Attributes, Plugin, PluginType, UpdateAuthority}
};
use crate::state::Config;
use crate::errors::StakingError;

// Constant for time calculations
const SECONDS_PER_DAY: i64 = 86400;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut, 
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    /// CHECK: NFT account will be checked by the mpl core program
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimRewards<'info> {
    pub fn claim_rewards(&mut self, bumps: &ClaimRewardsBumps) -> Result<()> {

        // Verify NFT owner and update authority
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(base_asset.owner == self.user.key(), StakingError::InvalidOwner);
        require!(base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()), StakingError::InvalidAuthority);
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(base_collection.update_authority == self.update_authority.key(), StakingError::InvalidAuthority);

        // Signer seeds for the update authority
        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        // Get current timestamp
        let current_timestamp = Clock::get()?.unix_timestamp;

        // Check if the NFT has the attribute plugin already added - return error if not
        let fetched_attribute_list = match fetch_plugin::<BaseAssetV1, Attributes>(&self.nft.to_account_info(), PluginType::Attributes) {
            Err(_) => {
                return Err(StakingError::NotStaked.into());
            }
            Ok((_, attributes, _)) => attributes,
        };

        // Extract and validate staking attributes
        let mut attribute_list: Vec<Attribute> = Vec::with_capacity(fetched_attribute_list.attribute_list.len());
        let mut staked_at_value: Option<&str> = None;
        let mut last_claimed_at_value: Option<&str> = None;
        
        for attribute in &fetched_attribute_list.attribute_list {
            match attribute.key.as_str() {
                "staked" => {
                    require!(attribute.value == "true", StakingError::NotStaked);
                    attribute_list.push(attribute.clone());
                }
                "staked_at" => {
                    staked_at_value = Some(&attribute.value);
                    attribute_list.push(attribute.clone());
                }
                "last_claimed_at" => {
                    last_claimed_at_value = Some(&attribute.value);
                    // Update last_claimed_at to current timestamp
                    attribute_list.push(Attribute { 
                        key: "last_claimed_at".to_string(), 
                        value: current_timestamp.to_string() 
                    });
                }
                _ => {
                    attribute_list.push(attribute.clone());
                }
            }
        }
        
        // Parse and validate staked_at
        let staked_at_timestamp = staked_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;
        
        // Verify NFT is currently staked
        require!(staked_at_timestamp != 0, StakingError::NotStaked);
        
        // Parse last_claimed_at (default to 0 if not found)
        let last_claimed_at_timestamp = last_claimed_at_value
            .unwrap_or("0")
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;
        
        // Calculate the start time for rewards (the later of staked_at or last_claimed_at)
        let reward_start_time = if last_claimed_at_timestamp > 0 {
            last_claimed_at_timestamp
        } else {
            staked_at_timestamp
        };
        
        // Calculate elapsed time since last reward period
        let elapsed_seconds = current_timestamp
            .checked_sub(reward_start_time)
            .ok_or(StakingError::InvalidTimestamp)?;
        
        let reward_time_days = elapsed_seconds
            .checked_div(SECONDS_PER_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;
        
        // Check that the freeze period has elapsed based on original staked_at time
        let total_staked_seconds = current_timestamp
            .checked_sub(staked_at_timestamp)
            .ok_or(StakingError::InvalidTimestamp)?;
        let total_staked_days = total_staked_seconds
            .checked_div(SECONDS_PER_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;
        
        require!(total_staked_days >= self.config.freeze_period as i64, StakingError::FreezePeriodNotElapsed);
        require!(reward_time_days > 0, StakingError::NoRewardsToClaim);

        // Calculate rewards based on time since last claim
        let amount = (reward_time_days as u64)
            .checked_mul(self.config.points_per_stake as u64)
            .ok_or(StakingError::Overflow)?;
        
        // Update the NFT attributes with new last_claimed_at
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes( Attributes { attribute_list }))
            .invoke_signed(&[signer_seeds])?;

        // Prepare signer seeds for config PDA
        let config_seeds = &[
            b"config",
            collection_key.as_ref(),
            &[self.config.config_bump],
        ];
        let config_signer_seeds = &[&config_seeds[..]];
        
        // Mint rewards tokens to user's ATA
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintToChecked {
            mint: self.rewards_mint.to_account_info(),
            to: self.user_rewards_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, config_signer_seeds);
        mint_to_checked(cpi_ctx, amount, self.rewards_mint.decimals)?;
        
        Ok(())
    }
}