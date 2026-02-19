use anchor_lang::prelude::*;
use crate::state::Oracle;
// Workaround from mpl-core types incompatibility with anchor 0.32.1
use crate::state::{OracleValidation, ExternalValidationResult};
// use mpl_core::types::{OracleValidation, ExternalValidationResult};

#[derive(Accounts)]
pub struct InitOracle<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = Oracle::INIT_SPACE,
        seeds = [b"oracle"],
        bump
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        seeds = [b"vault", oracle.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitOracle<'info> {
    pub fn init_oracle(&mut self, bumps: &InitOracleBumps) -> Result<()> {
        self.oracle.set_inner(Oracle { 
            validation: OracleValidation::V1 { 
                create: ExternalValidationResult::Pass, 
                transfer: ExternalValidationResult::Rejected, 
                burn: ExternalValidationResult::Pass, 
                update: ExternalValidationResult::Pass, 
            }, 
            bump: bumps.oracle,
            vault_bump: bumps.vault,
        });
        Ok(())
    }
}