use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Match {
    pub seed: u64,
    pub bet_amount: u64,
    pub match_duration: u32,
    pub player_A: Pubkey,
    pub player_B: Option<Pubkey>,
    pub created_at: i64,
    pub winner: Option<Pubkey>,
    pub status: Status,
    pub bump: u8
}

enum Status {
    Waiting,
    In_progress,
    Completed,
    Draw
}