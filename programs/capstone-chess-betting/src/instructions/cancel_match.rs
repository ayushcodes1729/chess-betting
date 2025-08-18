use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use anchor_lang::system_program::Transfer;

use crate::MatchState;
use crate::MatchConfig;
use crate::Status;

use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(code: String)]
pub struct CancelMatch<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

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

impl<'info> CancelMatch<'info> {
    pub fn cancel_match(&mut self, _code: String) -> Result<()> {
        let match_state = self.match_account.status;

        match match_state {
            Status::Waiting => {
                require_eq!(
                    self.player.key().to_string(),
                    self.match_account.player_a.key().to_string(),
                    ErrorCode::InvalidPlayerError
                );
                require_eq!(
                    self.vault.lamports(),
                    self.match_account.bet_amount,
                    ErrorCode::InvalidVaultBalanceError
                );

                let transfer_accounts = Transfer {
                    from: self.vault.to_account_info(),
                    to: self.player.to_account_info(),
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

                transfer(cpi_ctx, self.vault.lamports())?;
            }
            Status::InProgress => {
                let req_balance = self.match_account.bet_amount.checked_mul(2);
                require_eq!(
                    self.vault.lamports().to_string(),
                    req_balance.unwrap().to_string(),
                    ErrorCode::InvalidVaultBalanceError
                );
                let actor = self.player.key();
                let player_a = self.match_account.player_a.key();
                let player_b = self.match_account.player_b.unwrap().key();
                match (actor == player_a, actor == player_b) {
                    (true, false) => {
                        let transfer_accounts_1 = Transfer {
                            from: self.vault.to_account_info(),
                            to: self.player.to_account_info(),
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

                        let original_amount = self.match_account.bet_amount;
                        let final_amount_to_player = original_amount.checked_sub(original_amount.checked_div(100).unwrap());

                        transfer(cpi_ctx_2, original_amount)?;
                        transfer(cpi_ctx_1, final_amount_to_player.unwrap())?;
                    }
                    (false, true) => {
                        let transfer_accounts_1 = Transfer {
                            from: self.vault.to_account_info(),
                            to: self.player.to_account_info(),
                        };
                        let transfer_accounts_2 = Transfer {
                            from: self.vault.to_account_info(),
                            to: self.player_a.to_account_info(),
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

                        let original_amount = self.match_account.bet_amount;
                        let final_amount_to_player = original_amount.checked_sub(original_amount.checked_div(100).unwrap());

                        transfer(cpi_ctx_2, original_amount)?;
                        transfer(cpi_ctx_1, final_amount_to_player.unwrap())?;
                    }
                    (false, false) | (true, true) => {
                        return err!(ErrorCode::InvalidPlayerError);
                    }
                }
            }
            Status::Completed | Status::Draw => {
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
