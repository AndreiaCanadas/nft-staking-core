use anchor_lang::prelude::*;
use crate::state::Oracle;
use crate::constants::{SECONDS_IN_DAY, OPEN_HOUR, CLOSE_HOUR, REWARDS_TOLERANCE, REWARDS_LAMPORTS};
use crate::errors::StakingError;
use anchor_lang::system_program::{transfer, Transfer};

// Workaround from mpl-core types incompatibility with anchor 0.32.1
use crate::state::{OracleValidation, ExternalValidationResult};
// use mpl_core::types::{OracleValidation, ExternalValidationResult};

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"oracle"],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        seeds = [b"vault", oracle.key().as_ref()],
        bump = oracle.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> UpdateOracle<'info> {
    pub fn update_oracle(&mut self) -> Result<()> {

        let now = Clock::get()?.unix_timestamp;
        let seconds_since_midnight = now % SECONDS_IN_DAY;
        let is_open = seconds_since_midnight >= OPEN_HOUR && seconds_since_midnight < CLOSE_HOUR;

        if is_open {
            // Check if is already updated
            require!(self.oracle.validation == OracleValidation::V1 {
                create: ExternalValidationResult::Pass,
                transfer: ExternalValidationResult::Rejected,
                burn: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
            }, StakingError::AlreadyUpdated);
            // Update the oracle
            self.oracle.validation = OracleValidation::V1 {
                create: ExternalValidationResult::Pass,
                transfer: ExternalValidationResult::Approved,
                burn: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
            };
        } else {
            // Check if is already updated
            require!(self.oracle.validation == OracleValidation::V1 {
                create: ExternalValidationResult::Pass,
                transfer: ExternalValidationResult::Approved,
                burn: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
            }, StakingError::AlreadyUpdated);
            // Update the oracle
            self.oracle.validation = OracleValidation::V1 {
                create: ExternalValidationResult::Pass,
                transfer: ExternalValidationResult::Rejected,
                burn: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
            };
        }

        // Transfer rewards to the cranker if within tolerance
        let is_within_tolerance = 
            (seconds_since_midnight >= OPEN_HOUR && seconds_since_midnight < OPEN_HOUR + REWARDS_TOLERANCE) ||
            (seconds_since_midnight >= CLOSE_HOUR && seconds_since_midnight < CLOSE_HOUR + REWARDS_TOLERANCE);

        if is_within_tolerance {
            // signer seeds
            let oracle_key = self.oracle.key();
            let seeds = &[
                b"vault",
                oracle_key.as_ref(),
                &[self.oracle.vault_bump],
            ];
            let signer_seeds = &[&seeds[..]];
            let cpi_program = self.system_program.to_account_info();
            let cpi_accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.signer.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            transfer(cpi_ctx, REWARDS_LAMPORTS)?;
        }

        Ok(())
    }
}