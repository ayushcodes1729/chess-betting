use anchor_lang::prelude::*;

use crate::MatchConfig;

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
        self.config.set_inner(MatchConfig {
            authority: self.authority.key(),
            treasury_bump: bumps.treasury_pda,
            config_bump: bumps.config,
        });

        Ok(())
    }
}
