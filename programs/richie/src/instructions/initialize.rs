use anchor_lang::prelude::*;

use anchor_spl::token::{ Mint, Token, TokenAccount };

use crate::{ state::*, constants::* , error::RichieError };

#[derive(Accounts)]
pub struct InitializeStakeVault<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [CONFIG.as_bytes()],
        bump,
        space = 8 + Config::LEN
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub stake_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [VAULT.as_bytes()],
        bump,
        token::mint = stake_token_mint,
        token::authority = config
    )]
    pub stake_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeRewardVault<'info> {
    #[account(
       mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [REWARD.as_bytes()],
        bump,
        token::mint = reward_mint,
        token::authority = config,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = admin,
        seeds = [STAKE.as_bytes()],
        bump,
        space = 8 + Stakes::LEN,
    )]
    pub stakes: Box<Account<'info, Stakes>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageConfig<'info> {
    #[account(
       mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

pub fn initialize_stake_vault(
    ctx: Context<InitializeStakeVault>, 
    apr_bps: u64, 
    epoch_duration: i64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let clock = Clock::get()?;

    config.admin = ctx.accounts.admin.key();
    config.apr_bps = apr_bps;
    config.epoch_duration = epoch_duration;
    config.last_epoch_time = clock.unix_timestamp;

    config.stake_token_mint = ctx.accounts.stake_token_mint.key();
    config.stake_vault = ctx.accounts.stake_vault.key();

    // You may update these if you add them as inputs in the future
    config.reward_token_mint = Pubkey::default(); // Placeholder
    config.reward_vault = Pubkey::default();      // Placeholder

    config.total_staked = 0;
    config.index = 0;

    // Set default multipliers (100% base, 120%, 150%, etc.)
    config.multiplier = vec![100, 120, 150, 200, 300];

    Ok(())
}


pub fn initialize_reward_vault(
    ctx: Context<InitializeRewardVault>, 
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(ctx.accounts.admin.key() == config.admin, RichieError::UnAuthorized);
 
    config.reward_token_mint = ctx.accounts.reward_mint.key();
    config.reward_vault = ctx.accounts.reward_vault.key();

    Ok(())
}

pub fn update_epoch_duration(
    ctx: Context<ManageConfig>,
    duration: i64
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(ctx.accounts.admin.key() == config.admin, RichieError::UnAuthorized);
 
    config.epoch_duration = duration;
    
    Ok(())
}

pub fn update_multiplier(
    ctx: Context<ManageConfig>,
    new_multiplier: Vec<u64>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Check admin permission
    require_keys_eq!(
        ctx.accounts.admin.key(),
        config.admin,
        RichieError::UnAuthorized
    );

    // Optional: Limit number of multipliers
    require!(
        new_multiplier.len() <= Config::MAX_MULTIPLIERS,
        RichieError::TooManyMultipliers
    );

    // Update multiplier
    config.multiplier = new_multiplier;

    Ok(())
}

