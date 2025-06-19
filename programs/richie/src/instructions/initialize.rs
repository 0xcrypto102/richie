use anchor_lang::prelude::*;

use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{ state::*, constants::* };

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [CONFIG.as_bytes()],
        bump,
        space = 8 + Config::LEN
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [VAULT.as_bytes()],
        bump,
        token::mint = token_mint,
        token::authority = config
    )]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, apr_bps: u64, reward_wallet: Pubkey, reward_mint: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.admin = ctx.accounts.admin.key();
    config.apr_bps = apr_bps;
    config.reward_wallet = reward_wallet;
    config.reward_mint = reward_mint;

    Ok(())
}
