use anchor_lang::prelude::*;
use crate::state::Oracle;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::TransferV1CpiBuilder,
};

#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub new_owner: SystemAccount<'info>,
    #[account(
        seeds = [b"oracle"],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: Asset account will be checked by the mpl core program
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}
impl<'info> TransferNft<'info> {
    pub fn transfer_nft(&mut self) -> Result<()> {

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .new_owner(&self.new_owner.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .add_remaining_account(&self.oracle.to_account_info(), false, false)
            .invoke()?;

        Ok(())
    }
}