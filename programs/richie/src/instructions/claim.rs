use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Token, Mint, TokenAccount, Transfer}
};

use crate::{ constants::*, error::RichieError, state::* };

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user : Signer<'info>,
   
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [EPOCH.as_bytes(), &config.index.to_le_bytes()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,

    #[account(
        mut,
        seeds = [USER.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub stake_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [VAULT.as_bytes()],
        bump,
        token::mint = stake_token_mint,
        token::authority = config
    )]
    pub stake_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>, // user's $RICHIE

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user : Signer<'info>,
   
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [USER.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        seeds = [REWARD.as_bytes()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn claim(ctx: Context<Claim>) -> Result<()> {
    let user_stake = &mut ctx.accounts.user_stake;
    let amount = user_stake.pending_reward;
    require!(amount > 0, RichieError::NoReward);

    user_stake.pending_reward = 0;

    let (_, bump) = Pubkey::find_program_address(&[CONFIG.as_bytes()], ctx.program_id);
    let vault_seeds = &[CONFIG.as_bytes(), &[bump]];
    let signer = &[&vault_seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.reward_vault.to_account_info(),
        to: ctx.accounts.user_reward_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
    transfer(cpi_ctx, amount)?;

    Ok(())
}

pub fn withdraw(ctx: Context<Withdraw>, index: u64) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_stake = &mut ctx.accounts.user_stake;
    let stake_vault = &ctx.accounts.stake_vault;
    let to_token_account = &ctx.accounts.to_token_account;
    let epoch = &mut ctx.accounts.epoch;

    let clock = Clock::get()?;

    let current_index = config.index;
    let mut total_withdraw: u64 = 0;
    let mut total_penalty: u64 = 0;

    // Remove matching stake entries and sum withdrawable amount
    user_stake.stake_entries.retain(|entry| {
        if entry.last_staked_epoch_index == index {
            let end_epoch = entry.last_staked_epoch_index + entry.lock_period as u64;
            let mut unearned_curve = 0;
            if current_index < end_epoch {
                // Early withdrawal: 5% penalty
                let penalty = entry.amount * 5 / 100;
                total_penalty += penalty;
                total_withdraw += entry.amount - penalty;

                unearned_curve = entry.amount * (epoch.staked_end_time - clock.unix_timestamp) as u64 * entry.multiplier;
            } else {
                // No penalty
                total_withdraw += entry.amount;
                unearned_curve = entry.amount * (epoch.staked_end_time - clock.unix_timestamp) as u64;
            }
            epoch.total_curve -= unearned_curve;

            false // remove this entry
        } else {
            true // keep this entry
        }
    });

    require!(total_withdraw > 0, RichieError::NothingToWithdraw);

    // Transfer tokens from vault to user's token account
    let bump = ctx.bumps.config;
    let seeds = &[CONFIG.as_bytes(), &[bump]];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: stake_vault.to_account_info(),
        to: to_token_account.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
    anchor_spl::token::transfer(cpi_ctx, total_withdraw)?;

    Ok(())
}
