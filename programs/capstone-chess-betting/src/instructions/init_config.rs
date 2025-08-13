use anchor_lang::prelude::*;

use crate::MatchConfig;

use crate::error::ErrorCode;

const ADMIN_KEY: &str = "3CPvFJ3RDJH9RjpzY3WYxn1vcrL7J63hZQc9YboMaCg9";

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + MatchConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, MatchConfig>,

    #[account(
        mut,        //SystemAccount don't need to be initialized
        seeds = [b"treasury"],
        bump
    )]
    pub treasury_pda: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitConfig<'info> {
    pub fn init_config(&mut self, bumps: &InitConfigBumps) -> Result<()> {

        require_eq!(
            self.authority.key().to_string(),
            ADMIN_KEY.to_string(),
            ErrorCode::InvalidAdminError
        );

        self.config.set_inner(MatchConfig {
            authority: self.authority.key(),
            treasury_bump: bumps.treasury_pda,
            config_bump: bumps.config,
        });

        Ok(())
    }
}
