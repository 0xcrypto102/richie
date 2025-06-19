use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub apr_bps: u64, // APR in basis points (e.g., 1000 = 10%)
    pub reward_mint: Pubkey,
    pub reward_wallet: Pubkey,
}

impl Config {
    pub const LEN: usize = 32 + 8 + 32 + 32;
}
