use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MatchState{
    pub seed: u64, //done
    pub bet_amount: u64, // input
    pub match_duration: u32, // input
    pub player_a: Pubkey, // done
    pub player_b: Option<Pubkey>, //not in init
    pub created_at: i64, // clock
    pub winner: Option<Pubkey>, // not yet in init
    pub status: Status, // input
    pub bump: u8, // done
    pub vault_bump: u8 //done
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum Status {
    Waiting,
    InProgress,
    Completed,
    Draw
}

impl Space for Status {
    const INIT_SPACE: usize = 1; // size in bytes
}