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

pub fn stake(ctx: Context<Stake>, index: u64, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let user_stake = &mut ctx.accounts.user_stake;
    let config = &mut ctx.accounts.config;
    let epoch = &mut ctx.accounts.epoch;
    let stakes = &mut ctx.accounts.stakes;

    require!(clock.unix_timestamp >= epoch.staked_start_time, RichieError::InvalidStakeTime);
    require!(clock.unix_timestamp <= epoch.staked_start_time + epoch.stake_duration, RichieError::InvalidStakeTime);
    require!(index == config.index, RichieError::InvalidEpochIndex);

    if user_stake.status == true {
        user_stake.status = false;
        user_stake.user_curve = user_stake.amount * epoch.stake_duration as u64;
    }

    if user_stake.amount == 0 {
        user_stake.owner = ctx.accounts.user.key();
        user_stake.amount = amount;

        stakes.list.push(user_stake.key());
    } else {
        user_stake.amount += amount;
    }

    let available_time = epoch.stake_duration - (clock.unix_timestamp - epoch.staked_start_time); 
    
    epoch.total_curve += amount * available_time as u64;
    epoch.total_staked_amount += amount;

    user_stake.user_curve += amount * available_time as u64;

    user_stake.last_staked_time = clock.unix_timestamp;

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