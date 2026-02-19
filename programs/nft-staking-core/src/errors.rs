use anchor_lang::error_code;

#[error_code]
pub enum StakingError {
    #[msg("NFT owner key mismatch")]
    InvalidOwner,
    #[msg("Invalid update authority")]
    InvalidAuthority,
    #[msg("NFT already staked")]
    AlreadyStaked,
    #[msg("NFT not staked")]
    NotStaked,
    #[msg("Invalid timestamp value")]
    InvalidTimestamp,
    #[msg("NFT freeze period not elapsed")]
    FreezePeriodNotElapsed,
    #[msg("No rewards to claim")]
    NoRewardsToClaim,
    #[msg("Underflow")]
    Underflow,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid collection stats")]
    InvalidCollectionStats,
    #[msg("Oracle already updated")]
    AlreadyUpdated,
}