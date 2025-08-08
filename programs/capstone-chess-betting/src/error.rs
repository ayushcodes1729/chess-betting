use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Match is Invalid, check Status")]
    InvalidMatchError,

    #[msg("Insufficient balance in player's account")]
    InsufficientBalance,
    
    #[msg("Player A and B can't be same")]
    SamePlayerError
}
