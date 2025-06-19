use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{ state::*, constants::* };

#[derive(Accounts)]
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
    pub token_mint: Account<'info, Mint>,

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
        token::mint = token_mint,
        token::authority = config
    )]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let user_stake = &mut ctx.accounts.user_stake;

    // Update rewards before adding
    update_rewards(user_stake, clock.unix_timestamp, ctx.accounts.config.apr_bps)?;

    user_stake.amount += amount;
    user_stake.last_update = clock.unix_timestamp;

    // Transfer tokens
    let cpi_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.token_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    transfer(cpi_ctx, amount)?;

    Ok(())
}


fn update_rewards(user: &mut Account<UserStake>, now: i64, apr_bps: u64) -> Result<()> {
    let seconds_elapsed = (now - user.last_update).max(0) as u64;
    let yearly_seconds = 365 * 24 * 60 * 60;

    let earned = user.amount
        .checked_mul(apr_bps)
        .unwrap()
        .checked_mul(seconds_elapsed)
        .unwrap()
        / 10_000
        / yearly_seconds;

    user.pending_reward += earned;
    Ok(())
}