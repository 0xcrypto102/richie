use anchor_lang::prelude::*;

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub amount: u64,
    pub last_update: i64,
    pub pending_reward: u64,
}

impl UserStake {
    pub const LEN: usize = 32 + 8 + 8 + 8;
}
