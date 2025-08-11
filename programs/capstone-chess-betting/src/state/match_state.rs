use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MatchState{
    pub seed: u64, 
    pub bet_amount: u64, 
    pub match_duration: u32, 
    pub player_a: Pubkey, 
    pub player_b: Option<Pubkey>, 
    pub created_at: i64, 
    pub winner: Option<Pubkey>, 
    pub status: Status, 
    pub bump: u8, 
    pub vault_bump: u8 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Copy)]
pub enum Status {
    Waiting,
    InProgress,
    Completed,
    Draw
}

impl Space for Status {
    const INIT_SPACE: usize = 1; // size in bytes
}