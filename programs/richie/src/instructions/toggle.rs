use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{ state::*, constants::* , error::RichieError};

#[derive(Accounts)]
#[instruction(index: u64)]
pub struct Toggle<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = owner,
        seeds = [EPOCH.as_bytes(), &index.to_le_bytes()],
        bump,
        space = 8 + Epoch::LEN
    )]
    pub epoch: Account<'info, Epoch>,

    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,

    #[account(mut)]
    pub reward_mint_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [REWARD.as_bytes()],
        bump,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(index: u64)]
pub struct ManageStakerReward<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [EPOCH.as_bytes(), &index.to_le_bytes()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub user: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [USER.as_bytes(), user.key().as_ref()],
        bump,
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        seeds = [STAKE.as_bytes()],
        bump
    )]
    pub stakes: Account<'info, Stakes>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


pub fn toggle(ctx: Context<Toggle>, index: u64, reward_amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let config = &mut ctx.accounts.config;
    let epoch = &mut ctx.accounts.epoch;
    let owner = &mut ctx.accounts.owner;

    require!(owner.key() == config.admin, RichieError::UnAuthorized);
    require!(index == config.index, RichieError::InvalidEpochIndex); // we use index 0 as staking before first epoch
    config.index += 1;

    let mut duration = 0;

    if index == 0 {
        duration = 6 * 60 * 60; // 6 hours
        require!(reward_amount == 0, RichieError::InvalidRewardAmount);
    } else {
        duration = config.epoch_duration;
        require!(reward_amount > 0, RichieError::InvalidRewardAmount);
    }

    epoch.index = index;
    epoch.staked_start_time = clock.unix_timestamp;
    epoch.stake_duration = duration;
    epoch.staked_end_time = epoch.staked_start_time + duration;

    if index > 0 {
        // Transfer tokens
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_mint_token_account.to_account_info(),
            to: ctx.accounts.reward_vault.to_account_info(),
            authority: owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, reward_amount)?;
    }

    epoch.reward = reward_amount;
    epoch.total_staked_amount = config.total_staked;
    epoch.total_curve = config.total_curve;
    config.total_curve = 0;
    
    Ok(())
}

pub fn manage_staker_reward(ctx: Context<ManageStakerReward>, index: u64) -> Result<()> {
    let clock = Clock::get()?;
    let config = &mut ctx.accounts.config;
    let epoch = &mut ctx.accounts.epoch;
    let user_stake = &mut ctx.accounts.user_stake;
    let stakes = &ctx.accounts.stakes;
    let admin = &ctx.accounts.admin;

    // Validation
    require!(stakes.list.contains(&user_stake.key()), RichieError::InvalidUserStake);
    require!(config.index == index, RichieError::InvalidEpochIndex);
    require!(admin.key() == config.admin, RichieError::UnAuthorized);
    require!(
        epoch.staked_start_time + epoch.stake_duration < clock.unix_timestamp,
        RichieError::UnFinishedEpoch
    );

    let duration = config.epoch_duration;

    let mut reward_sum: u64 = 0;
    for entry in user_stake.stake_entries.iter_mut() {
        if entry.calculated_index == index {
            msg!("It was already calculated")
        } else {
            if entry.last_staked_epoch_index + entry.lock_period as u64 - 1 >= index {
                let reward_share = (entry.boosted_curve as u128)
                    .checked_mul(epoch.reward as u128)
                    .unwrap_or(0)
                    .checked_div(epoch.total_curve as u128)
                    .unwrap_or(0) as u64;

                reward_sum += reward_share;

                entry.base_curve = entry.amount * duration as u64;
                entry.boosted_curve = entry.base_curve * entry.multiplier;
            } else {
                let reward_share = (entry.base_curve as u128)
                    .checked_mul(epoch.reward as u128)
                    .unwrap_or(0)
                    .checked_div(epoch.total_curve as u128)
                    .unwrap_or(0) as u64;

                reward_sum += reward_share;

                entry.base_curve = entry.amount * duration as u64;
                entry.boosted_curve = entry.base_curve;
            }
            config.total_curve += entry.boosted_curve;
            entry.calculated_index = index;
        }
    }

    let is_epoch_zero = epoch.index == 0;

    if !is_epoch_zero {
        user_stake.pending_reward = user_stake.pending_reward.saturating_add(reward_sum);
        if !epoch.claimable {
            epoch.claimable = true;
        }
    } else {
        config.total_curve = config.total_staked * config.epoch_duration as u64;
    }

    Ok(())
}