use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use anchor_lang::system_program::Transfer;

use crate::MatchState;
use crate::Status::*;

use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(code: String)]
pub struct AcceptMatch<'info> {
    #[account(mut)]
    pub player_b: Signer<'info>,

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

    pub system_program: Program<'info, System>,
}

impl<'info> AcceptMatch<'info> {
    pub fn accept_match(&mut self, _code: String) -> Result<()> {
        require!(
            self.match_account.status == Waiting,
            ErrorCode::InvalidMatchError
        );
        require_neq!(
            self.match_account.player_a.key().to_string(),
            self.player_b.key().to_string(),
            ErrorCode::SamePlayerError
        );
        require!(
            self.player_b.get_lamports() > self.match_account.bet_amount,
            ErrorCode::InsufficientBalance
        );

        self.match_account.player_b = Some(self.player_b.key());

        let transfer_accounts = Transfer {
            from: self.player_b.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(cpi_ctx, self.match_account.bet_amount)?;

        self.match_account.status = InProgress;

        Ok(())
    }
}
