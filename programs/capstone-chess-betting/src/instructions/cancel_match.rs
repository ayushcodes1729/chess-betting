use anchor_lang::prelude::*;

use crate::MatchState;
use crate::Status;
use crate::Status::*;

use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(code: String)]
pub struct CancelMatch<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [b"match", match_account.seed.to_be_bytes().as_ref(), code.as_bytes(), match_account.player_a.key().as_ref()],
        bump = match_account.bump,
    )]
    pub match_account: Account<'info, MatchState>,

    #[account(
        mut,
        seeds = [b"vault", match_account.key().as_ref()],
        bump = match_account.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,        //SystemAccount don't need to be initialized
        seeds = [b"treasury"],
        bump
    )]
    pub treasury_pda: SystemAccount<'info>,

    pub system_program: Program<'info, System>, // we need it anyways for the cpi(since we are transferring native sols)
}

impl <'info> CancelMatch<'info> {
    pub fn cancel_match(&mut self) -> Result<()> {
        let match_state = self.match_account.status;

        match match_state {
            Status::Waiting => {
                require_eq!(self.player.key().to_string(), self.match_account.player_a.key().to_string(), ErrorCode::InvalidPlayerError);
            }
            Status::InProgress => {

            }
            Status::Completed | Status::Draw => {
                ErrorCode::InvalidMatchError;
            }
        }
        Ok(())
    }
}