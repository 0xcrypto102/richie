use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub apr_bps: u64,
    pub epoch_duration: i64,
    pub last_epoch_time: i64,
    pub stake_token_mint: Pubkey,
    pub stake_vault: Pubkey,
    pub reward_token_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub total_staked: u64,
    pub index: u64,  // the index of epoch
}

impl Config {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 32 + 32 +8 + 8;
}
