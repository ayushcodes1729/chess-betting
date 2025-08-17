use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program::transfer;
use anchor_lang::system_program::Transfer;

use crate::MatchConfig;
use crate::MatchState;
use crate::Status;
use crate::Status::*;

use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(code: String)]
pub struct FinalPayments<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub player_a: SystemAccount<'info>,

    #[account(mut)]
    pub player_b: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"match", match_account.seed.to_le_bytes().as_ref(), code.as_bytes(), match_account.player_a.key().as_ref()],
        bump = match_account.bump,
        close = treasury_pda
    )]
    pub match_account: Account<'info, MatchState>,

    #[account(
        mut,
        seeds = [b"vault", match_account.key().as_ref()],
        bump = match_account.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.config_bump
    )]
    pub config: Account<'info, MatchConfig>,

    #[account(
        mut,        //SystemAccount don't need to be initialized
        seeds = [b"treasury"],
        bump = config.treasury_bump
    )]
    pub treasury_pda: SystemAccount<'info>,

    pub system_program: Program<'info, System>, // we need it anyways for the cpi(since we are transferring native sols)
}

impl<'info> FinalPayments<'info> {
    pub fn final_payouts(&mut self, _code: String, winner_key: Option<Pubkey>) -> Result<()> {
        if winner_key != None {
            self.match_account.status = Completed;
        } else {
            self.match_account.status = Draw;
        }

        let match_state = self.match_account.status;

        match match_state {
            Status::Completed => {
                let req_balance = self.match_account.bet_amount.checked_mul(2);
                require!(
                    winner_key.unwrap().key().to_string() == self.player_a.key().to_string()
                        || winner_key.unwrap().key().to_string() == self.player_b.key().to_string(),
                    ErrorCode::InvalidPlayerError
                );
                require_eq!(
                    self.vault.lamports().to_string(),
                    req_balance.unwrap().to_string(),
                    ErrorCode::InvalidVaultBalanceError
                );
                self.match_account.winner = Some(winner_key.unwrap().key());

                let bet = self.match_account.bet_amount;
                let total_bet_amount = bet.checked_mul(2).unwrap();
                let winning_amount: u64;

                if bet <= LAMPORTS_PER_SOL {
                    winning_amount = total_bet_amount
                        .checked_sub(total_bet_amount.checked_mul(50).unwrap().checked_div(10_000).unwrap())
                        .unwrap(); // 0.5% of 2* bet_ammount
                } else if LAMPORTS_PER_SOL < bet && bet <= LAMPORTS_PER_SOL * 5 {
                    winning_amount = total_bet_amount
                        .checked_sub(total_bet_amount.checked_mul(100).unwrap().checked_div(10_000).unwrap())
                        .unwrap(); // 1% of 2* bet_ammount
                } else if bet > LAMPORTS_PER_SOL * 5 {
                    winning_amount = total_bet_amount
                        .checked_sub(total_bet_amount.checked_mul(150).unwrap().checked_div(10_000).unwrap())
                        .unwrap(); // 1.5% of 2* bet_ammount
                } else {
                    return err!(ErrorCode::InvalidBetAmount);
                }

                match (
                    winner_key.unwrap() == self.player_a.key(),
                    winner_key.unwrap() == self.player_b.key(),
                ) {
                    (true, false) => {
                        let transfer_accounts = Transfer {
                            from: self.vault.to_account_info(),
                            to: self.player_a.to_account_info(),
                        };

                        let signer_seeds: &[&[&[u8]]; 1] = &[&[
                            b"vault",
                            self.match_account.to_account_info().key.as_ref(), //temporary value dropped while borrowed, that's why removed () after key
                            &[self.match_account.vault_bump],
                        ]];

                        let cpi_ctx = CpiContext::new_with_signer(
                            self.system_program.to_account_info(),
                            transfer_accounts,
                            signer_seeds,
                        );

                        transfer(cpi_ctx, winning_amount)?;
                    }
                    (false, true) => {
                        let transfer_accounts = Transfer {
                            from: self.vault.to_account_info(),
                            to: self.player_b.to_account_info(),
                        };

                        let signer_seeds: &[&[&[u8]]; 1] = &[&[
                            b"vault",
                            self.match_account.to_account_info().key.as_ref(), //temporary value dropped while borrowed, that's why removed () after key
                            &[self.match_account.vault_bump],
                        ]];

                        let cpi_ctx = CpiContext::new_with_signer(
                            self.system_program.to_account_info(),
                            transfer_accounts,
                            signer_seeds,
                        );

                        transfer(cpi_ctx, winning_amount)?;
                    }
                    (true, true) | (false, false) => {
                        return err!(ErrorCode::InvalidWinnerError);
                    }
                }
            }
            Status::Draw => {
                let req_balance = self.match_account.bet_amount.checked_mul(2);
                require_eq!(
                    self.vault.lamports().to_string(),
                    req_balance.unwrap().to_string(),
                    ErrorCode::InvalidVaultBalanceError
                );

                let bet = self.match_account.bet_amount;
                let draw_amount = bet
                    .checked_sub(bet.checked_mul(100).unwrap().checked_div(10_000).unwrap())
                    .unwrap(); // each person will get there money refunded by 1%

                let transfer_accounts_1 = Transfer {
                    from: self.vault.to_account_info(),
                    to: self.player_a.to_account_info(),
                };
                let transfer_accounts_2 = Transfer {
                    from: self.vault.to_account_info(),
                    to: self.player_b.to_account_info(),
                };

                let signer_seeds: &[&[&[u8]]; 1] = &[&[
                    b"vault",
                    self.match_account.to_account_info().key.as_ref(), //temporary value dropped while borrowed, that's why removed () after key
                    &[self.match_account.vault_bump],
                ]];

                let cpi_ctx_1 = CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    transfer_accounts_1,
                    signer_seeds,
                );
                let cpi_ctx_2 = CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    transfer_accounts_2,
                    signer_seeds,
                );

                transfer(cpi_ctx_1, draw_amount)?;
                transfer(cpi_ctx_2, draw_amount)?;
            }
            Status::InProgress | Status::Waiting => {
                return err!(ErrorCode::InvalidMatchError);
            }
        }

        // Sweep any leftover lamports from the vault to the treasury via CPI
        let leftover = self.vault.lamports();
        if leftover > 0 {
            let transfer_accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.treasury_pda.to_account_info(),
            };

            let signer_seeds: &[&[&[u8]]; 1] = &[&[
                b"vault",
                self.match_account.to_account_info().key.as_ref(),
                &[self.match_account.vault_bump],
            ]];

            let cpi_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                transfer_accounts,
                signer_seeds,
            );

            // It’s fine to transfer the full balance for a 0-space SystemAccount
            transfer(cpi_ctx, leftover)?;
        }

        Ok(())
    }
}
