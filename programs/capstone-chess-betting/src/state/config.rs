use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MatchConfig {
    pub authority: Pubkey,
    pub treasury_bump: u8,
    pub config_bump: u8
}