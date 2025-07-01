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
    pub total_curve: u64,
    pub index: u64,
    pub multiplier: Vec<u64>, // New field: multiplier per lock period (e.g., 1,2,4,8,16)
}

impl Config {
    pub const MAX_MULTIPLIERS: usize = 5;

    pub const LEN: usize = 
        8 +                      // discriminator
        32 +                    // admin
        8 +                     // apr_bps
        8 +                     // epoch_duration
        8 +                     // last_epoch_time
        32 +                    // stake_token_mint
        32 +                    // stake_vault
        32 +                    // reward_token_mint
        32 +                    // reward_vault
        8 +                     // total_staked
        8 +                     // index
        4 + 8 * Self::MAX_MULTIPLIERS; // multiplier vec: 4-byte prefix + 8 bytes per entry
}
