use anchor_lang::prelude::*;
// use anchor_spl::token_interface::{Mint, TokenInterface};
use crate::state::Config;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init, 
        payer = authority, 
        space = 8 + Config::INIT_SPACE, 
        seeds = [b"config"], 
        bump
    )]
    pub config: Account<'info, Config>,
    // #[account(
    //     init,
    //     payer = authority,
    //     mint::decimals = 6,
    //     mint::authority = config,
    //     seeds = [b"rewards", config.key().as_ref()],
    //     bump
    // )]
    // pub rewards_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    // pub token_program: Interface<'info, TokenInterface>,
}
impl Initialize<'_> {
    pub fn init_config(&mut self, points_per_stake: u8, freeze_period: u8, bumps: &InitializeBumps) -> Result<()> {
        self.config.set_inner(Config { 
            authority: self.authority.key(), 
            points_per_stake, 
            freeze_period, 
            rewards_bump: 0, 
            config_bump: bumps.config });
        Ok(())
    }
}