use anchor_lang::prelude::*;

mod state;
mod instructions;
use instructions::*;

declare_id!("7WvxBTMfM9ySNJsp3qgzw2pKLjmVskmgUZECPkenG5uw");

#[program]
pub mod nft_staking_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
