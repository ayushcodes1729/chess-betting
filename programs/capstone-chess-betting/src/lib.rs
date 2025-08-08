#![allow(deprecated)]
#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3fsvNChoZg4zdod2Tfw8ApdPhgShnmpJDrfe1GPweHdm");

#[program]
pub mod capstone_chess_betting {
    use super::*;

    pub fn initialize_match(
        ctx: Context<InitializeMatch>,
        match_duration: u32,
        bet_amount: u64,
        player_b: Option<Pubkey>,
        winner: Option<Pubkey>,
        seed: u64,
        code: String
    ) -> Result<()> {
        ctx.accounts.init_match(match_duration, bet_amount, player_b, winner, seed, code.clone(), &ctx.bumps)?;
        ctx.accounts.deposit_bet()?;

        Ok(())
    }
}
