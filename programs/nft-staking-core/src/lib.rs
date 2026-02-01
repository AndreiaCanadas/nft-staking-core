use anchor_lang::prelude::*;

mod state;
mod instructions;
mod errors;
use instructions::*;

declare_id!("7WvxBTMfM9ySNJsp3qgzw2pKLjmVskmgUZECPkenG5uw");

#[program]
pub mod nft_staking_core {
    use super::*;

    pub fn create_collection(ctx: Context<CreateCollection>, name: String, uri: String) -> Result<()> {
        ctx.accounts.create_collection(name, uri, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<Mint>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_nft(name, uri, &ctx.bumps)
    }

    pub fn initialize_config(ctx: Context<Initialize>, points_per_stake: u8, freeze_period: u8) -> Result<()> {
        ctx.accounts.init_config(points_per_stake, freeze_period, &ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }

}
