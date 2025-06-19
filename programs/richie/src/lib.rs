pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
use instructions::*;
pub use state::*;

declare_id!("DBCxLXexkfhav6oN9vrAmPW2ae6MQzV3HikTzFR4uaGa");

#[program]
pub mod richie {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,apr_bps: u64, reward_wallet: Pubkey, reward_mint: Pubkey) -> Result<()> {
        initialize::initialize(ctx, apr_bps, reward_wallet, reward_mint)
    }
}
