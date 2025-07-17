pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
use instructions::*;
pub use state::*;

declare_id!("13PntzJFX9da8hCrAmB9q5HcMYfPeQ4HMYcNU56qqBrF");

#[program]
pub mod richie {
    use super::*;

    pub fn initialize_stake_vault(
        ctx: Context<InitializeStakeVault>,
        apr_bps: u64, 
        epoch_duration: i64
    ) -> Result<()> {
        instructions::initialize_stake_vault(ctx, apr_bps, epoch_duration)
    }

    pub fn initialize_reward_vault(
        ctx: Context<InitializeRewardVault>,
    ) -> Result<()> {
        instructions::initialize_reward_vault(ctx)
    }

    pub fn update_epoch_duration(
        ctx: Context<ManageConfig>,
        duration: i64
    ) -> Result<()> {
        instructions::update_epoch_duration(ctx, duration)
    }

    pub fn update_multiplier(
        ctx: Context<ManageConfig>,
        new_multiplier: Vec<u64>
    ) -> Result<()> {
        instructions::update_multiplier(ctx, new_multiplier)
    }

    pub fn toggle(
        ctx: Context<Toggle>,
        index: u64,
        reward_amount: u64
    ) -> Result<()> {
        instructions::toggle(ctx, index, reward_amount)
    }

    pub fn manage_staker_reward(
        ctx: Context<ManageStakerReward>,
        index: u64
    ) -> Result<()> {
        instructions::manage_staker_reward(ctx, index)
    }

    pub fn stake(
        ctx: Context<Stake>,
        index: u64,
        amount: u64,
        lock_period: u8
    ) -> Result<()> {
        instructions::stake(ctx, index, amount, lock_period)
    }

    pub fn claim(
        ctx: Context<Claim>
    ) -> Result<()> {
        instructions::claim(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>, index: u64) -> Result<()> {
        instructions::withdraw(ctx, index)
    }
}
