use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{ state::*, constants::* , error::RichieError};

#[derive(Accounts)]
#[instruction(index: u64)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub stake_token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [USER.as_bytes(), user.key().as_ref()],
        bump,
        space = 8 + UserStake::LEN
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>, // user's $RICHIE

    #[account(
        mut,
        seeds = [VAULT.as_bytes()],
        bump,
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [EPOCH.as_bytes(), &index.to_le_bytes()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,

    #[account(
        mut,
        seeds = [STAKE.as_bytes()],
        bump
    )]
    pub stakes: Account<'info, Stakes>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn stake(ctx: Context<Stake>, index: u64, amount: u64, lock_period: u8) -> Result<()> {
    let clock = Clock::get()?;
    let user_stake = &mut ctx.accounts.user_stake;
    let config = &mut ctx.accounts.config;
    let epoch = &mut ctx.accounts.epoch;
    let stakes = &mut ctx.accounts.stakes;

    if index == 0 {
        // Pre-epoch staking allowed any time with lock_period = 1
        require!(lock_period == 1, RichieError::InvalidLockPeriod);
    } else {
        // Normal staking logic for active epochs
        require!(index == config.index, RichieError::InvalidEpochIndex);
        require!(
            clock.unix_timestamp >= epoch.staked_start_time &&
            clock.unix_timestamp <= epoch.staked_start_time + epoch.stake_duration,
            RichieError::InvalidStakeTime
        );
    }
    require!(index == config.index, RichieError::InvalidEpochIndex);

    if user_stake.stake_entries.is_empty() {
        user_stake.owner = ctx.accounts.user.key();
        stakes.list.push(user_stake.key());
    }

    let (base_curve, boosted_curve, multiplier) = if index == 0 {
        // Skip curve calculation for pre-epoch stake
        (0, 0, 0)
    } else {
        let available_time = epoch.stake_duration - (clock.unix_timestamp - epoch.staked_start_time);
        let multiplier = get_multiplier(config, lock_period)?;
        let base_curve = amount * available_time as u64;
        let boosted_curve = base_curve * multiplier / 100;
        (base_curve, boosted_curve, multiplier)
    };

    // Append new stake entry
    if index == 0 {
        if let Some(entry) = user_stake
            .stake_entries
            .iter_mut()
            .find(|e| e.last_staked_epoch_index == 0)
        {
            entry.amount += amount;
            // Optional: update base/boosted_curve if you want to accumulate (but likely 0 for epoch 0)
        } else {
            user_stake.stake_entries.push(StakeEntry {
                amount,
                last_staked_epoch_index: index,
                lock_period,
                multiplier,
                base_curve,
                boosted_curve,
                calculated_index: 0,
            });
        }
    } else {
        user_stake.stake_entries.push(StakeEntry {
            amount,
            last_staked_epoch_index: index,
            lock_period,
            multiplier,
            base_curve,
            boosted_curve,
            calculated_index: 0,
        });
    }

    // Update epoch stats
    if index != 0 {
        epoch.total_curve += boosted_curve;
        epoch.total_staked_amount += amount;
    }
    // Transfer tokens
    let cpi_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.stake_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    transfer(cpi_ctx, amount)?;

    config.total_staked += amount;

    Ok(())
}

fn get_multiplier(config: &Config, lock_period: u8) -> Result<u64> {
    match lock_period {
        1 => Ok(*config.multiplier.get(0).ok_or(RichieError::InvalidLockPeriod)?),
        2 => Ok(*config.multiplier.get(1).ok_or(RichieError::InvalidLockPeriod)?),
        4 => Ok(*config.multiplier.get(2).ok_or(RichieError::InvalidLockPeriod)?),
        8 => Ok(*config.multiplier.get(3).ok_or(RichieError::InvalidLockPeriod)?),
        16 => Ok(*config.multiplier.get(4).ok_or(RichieError::InvalidLockPeriod)?),
        _ => Err(RichieError::InvalidLockPeriod.into()),
    }
}