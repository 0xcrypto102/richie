use std::sync::atomic::AtomicBool;

use anchor_lang::prelude::*;

#[account]
pub struct Stakes {
    pub list: Vec<Pubkey>,
}

impl Stakes {
    pub const MAX_USERS: usize = 300; // Set as needed
    pub const LEN: usize = 4 + (32 * Self::MAX_USERS); // 4 bytes for vector prefix + 32 bytes per Pubkey
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub stake_entries: Vec<StakeEntry>,
    pub pending_reward: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StakeEntry {
    pub amount: u64,
    pub last_staked_epoch_index: u64,
    pub lock_period: u8,
    pub multiplier: u64,
    pub base_curve: u64,
    pub boosted_curve: u64,
    pub calculated_index: u64,
}

impl UserStake {
    pub const MAX_ENTRIES: usize = 20;

    pub const LEN: usize =
        32 +                            // owner
        4 + (8 + 8 + 1 + 8 + 8 + 8) * Self::MAX_ENTRIES + // Vec<StakeEntry>: 4 bytes + N * entry size
        8;                              // pending_reward
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
    pub claimable: bool
}

impl Epoch {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
}