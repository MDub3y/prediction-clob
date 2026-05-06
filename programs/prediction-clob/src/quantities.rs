use anchor_lang::prelude::*;

// smallest unit of price movement
#[zero_copy]
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticks {
    pub val: u64,
}

// smallest unit of outcome token
#[zero_copy]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct BaseLots {
    pub val: u64,
}

// smallest unit of collateral
#[zero_copy]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct QuoteLots {
    pub val: u64,
}
