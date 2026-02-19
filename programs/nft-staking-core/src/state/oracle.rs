use anchor_lang::prelude::*;

// Workaround from mpl-core types incompatibility with anchor 0.32.1
// use mpl_core::types::{OracleValidation, ExternalValidationResult};
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExternalValidationResult {
    Approved,
    Rejected,
    Pass,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OracleValidation {
    Uninitialized,
    V1 {
        create: ExternalValidationResult,
        transfer: ExternalValidationResult,
        burn: ExternalValidationResult,
        update: ExternalValidationResult,
    },
}

// Permissionless oracle account to save the Approved/Rejected per lifecycle event
#[account]
pub struct Oracle {
    pub validation: OracleValidation,   // Oracle validation structure (approved/rejected per lifecycle event)
    pub bump: u8,                       // bump seed for the oracle account PDA
    pub vault_bump: u8,                 // bump seed for the vault account PDA
}
impl Space for Oracle {
    const INIT_SPACE: usize = 8 + 5 + 1 + 1;
}