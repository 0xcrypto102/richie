use anchor_lang::prelude::*;

#[account]
pub struct Stakes {
    pub list: Vec<Pubkey>,
}

impl Stakes {
    pub const MAX_USERS: usize = 1000; // Set as needed
    pub const LEN: usize = 4 + (32 * Self::MAX_USERS); // 4 bytes for vector prefix + 32 bytes per Pubkey
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub amount: u64,
    pub last_staked_time: i64,
    pub user_curve: u64,
    pub pending_reward: u64,
    pub status: bool,
}

impl UserStake {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 1;
}

#[account]
pub struct Epoch {
    pub index: u64,
    pub staked_start_time: i64,
    pub stake_duration: i64,
    pub staked_end_time: i64,
    pub reward: u64,
    pub total_curve: u64,
    pub total_staked_amount: u64,
}

impl Epoch {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8 + 8 + 8;
}