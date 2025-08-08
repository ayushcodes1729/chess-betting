use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use anchor_lang::system_program::Transfer;

use crate::MatchState;
use crate::Status::*;

use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(seed: u64, code: String)]
pub struct InitializeMatch<'info> {
    #[account(mut)]
    pub player_a: Signer<'info>,

    #[account(
        init,
        payer = player_a,
        space = 8 + MatchState::INIT_SPACE,
        seeds = [b"match", seed.to_le_bytes().as_ref(), code.as_bytes(), player_a.key().as_ref()],
        bump,
    )]
    pub match_account: Account<'info, MatchState>,

    #[account(
        mut,
        seeds = [b"vault", match_account.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMatch<'info> {
    pub fn init_match(
        &mut self,
        match_duration: u32,
        bet_amount: u64,
        player_b: Option<Pubkey>,
        winner: Option<Pubkey>,
        seed: u64,
        _code: String,
        bumps: &InitializeMatchBumps,
    ) -> Result<()> {
        self.match_account.set_inner(MatchState {
            seed,
            bet_amount,
            match_duration,
            player_a: self.player_a.key(),
            player_b,
            created_at: Clock::get()?.unix_timestamp, // handle this in ER if u add that or handle it off-chain that match ends after the duration
            winner,
            status: Waiting,
            bump: bumps.match_account,
        });

        Ok(())
    }

    pub fn deposit_bet(&mut self) -> Result<()> {
        require!(self.match_account.status == Waiting, ErrorCode::InvalidMatchError);
        
        let transfer_accounts = Transfer {
            from: self.player_a.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(cpi_ctx, self.match_account.bet_amount)?;
        Ok(())
    }
}
