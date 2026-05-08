use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The orderbook is full.")]
    BookFull,
    #[msg("Market order could not be filled within slippage limits.")]
    SlippageExceeded,
    #[msg("Math overflow.")]
    MathOverflow,
    #[msg("No funds available to claim.")]
    NothingToClaim,
}
