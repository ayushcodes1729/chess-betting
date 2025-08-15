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
        seed: u64,
        code: String,
        match_duration: u32,
        bet_amount: u64,
        player_b: Option<Pubkey>,
        winner: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.init_match(
            seed,
            code,
            match_duration,
            bet_amount,
            player_b,
            winner,
            &ctx.bumps,
        )?;
        ctx.accounts.deposit_bet()?;

        Ok(())
    }

    pub fn init_config(
        ctx: Context<InitConfig>,
    ) -> Result<()> {
        ctx.accounts.init_config(&ctx.bumps)?;
        Ok(())
    }

    pub fn accept_match(ctx: Context<AcceptMatch>, code: String) -> Result<()>{
        ctx.accounts.accept_match(code)?;
        Ok(())
    }

    pub fn cancel_match(ctx: Context<CancelMatch>, code: String) -> Result<()>{
        ctx.accounts.cancel_match(code)?;
        Ok(())
    }

    pub fn final_payouts(ctx: Context<FinalPayments>, code: String, winner_key: Option<Pubkey>) -> Result<()> {
        ctx.accounts.final_payouts(code, winner_key)?;
        Ok(())
    }

    pub fn withdraw_from_treasury(ctx: Context<WithdrawTreasury>) -> Result<()>{
        ctx.accounts.withdraw_from_treasury()?;
        Ok(())
    }
}
