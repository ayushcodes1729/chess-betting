use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Match is Invalid, check Status")]
    InvalidMatchError,

    #[msg("Insufficient balance in player's account")]
    InsufficientBalance,

    #[msg("Invalid balance in vault pda")]
    InvalidVaultBalanceError,
    
    #[msg("This player is not expected here")]
    InvalidPlayerError,
    
    #[msg("Player A and B can't be same")]
    SamePlayerError
}
