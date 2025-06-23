use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{ constants::*, error::RichieError, state::* };

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
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
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
