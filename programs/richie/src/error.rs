use anchor_lang::prelude::*;

#[error_code]
pub enum RichieError {
    #[msg("UnAuthorized.")]
    UnAuthorized,
    #[msg("Too many multipliers provided")]
    TooManyMultipliers,
    #[msg("Not enough tokens staked.")]
    InsufficientStake,
    #[msg("Epoch duration has not passed yet.")]
    EpochTooSoon,
    #[msg("No reward to claim.")]
    NoReward,
    #[msg("This is invalid time to stake.")]
    InvalidStakeTime,
    #[msg("This is invalid epoch index.")]
    InvalidEpochIndex,
    #[msg("This is invalid user stake account.")]
    InvalidUserStake,
    #[msg("The reward was already calculated.")]
    AlreadyCalculated,
    #[msg("The epoch is not finished yet.")]
    UnFinishedEpoch,
    #[msg("Invalid lock period")]
    InvalidLockPeriod,
    #[msg("No stake available for withdrawal.")]
    NothingToWithdraw,
}