use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::MatchConfig;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"config"],
        bump = config.config_bump,
        has_one = authority @ ErrorCode::InvalidAdminError
    )]
    pub config: Account<'info, MatchConfig>,

    #[account(
        mut,        //SystemAccount don't need to be initialized
        seeds = [b"treasury"],
        bump = config.treasury_bump,
    )]
    pub treasury_pda: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl <'info> WithdrawTreasury<'info> {
    pub fn withdraw_from_treasury(&mut self) -> Result<()> {
        let transfer_accounts = Transfer {
            from: self.treasury_pda.to_account_info(),
            to: self.authority.to_account_info()
        };

        let signer_seeds: &[&[&[u8]]; 1] = &[&[
            b"treasury",
            &[self.config.treasury_bump]
        ]];

        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), transfer_accounts, signer_seeds);

        let rent_min = Rent::get()?.minimum_balance(self.treasury_pda.data_len());

        let transfer_amount = self.treasury_pda.lamports().saturating_sub(rent_min);

        transfer(cpi_ctx, transfer_amount)?;

        Ok(())
    }
}