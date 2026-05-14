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
    #[msg("The market has already been settled.")]
    MarketAlreadySettled,
    #[msg("The event has not reached its settlement deadline yet.")]
    MarketNotExpired,
    #[msg("The challenge period is still active.")]
    ChallengePeriodActive,
    #[msg("Unauthorized: You do not own this order.")]
    Unauthorized,
    #[msg("The order is not in an open state.")]
    OrderNotOpen,

    #[msg("Insufficient collateral balance in ledger. Please deposit USDC.")]
    InsufficientCollateral,
    #[msg("Insufficient share balance in ledger. Please split or buy shares.")]
    InsufficientShares,
    #[msg("The provided price is invalid.")]
    InvalidPrice,
}
