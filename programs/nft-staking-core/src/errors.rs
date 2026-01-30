use anchor_lang::error_code;

#[error_code]
pub enum StakingError {
    #[msg("NFT owner key mismatch")]
    InvalidOwner,
    #[msg("Invalid update authority")]
    InvalidAuthority,
    #[msg("NFT already staked")]
    AlreadyStaked,
}