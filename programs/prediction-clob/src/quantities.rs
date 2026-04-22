use anchor_lang::prelude::*;

// smallest unit of price movement
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, InitSpace)]
pub struct Ticks (pub u64);

// smallest unit of outcome token
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub struct BaseLots(pub u64);

// smallest unit of collateral
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub struct QuoteLots(pub u64);