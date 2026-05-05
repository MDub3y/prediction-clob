use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

// smallest unit of price movement
#[zero_copy]
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticks {
    pub val: u64,
}

#[zero_copy]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct BaseLots {
    pub val: u64,
}

#[zero_copy]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct QuoteLots {
    pub val: u64,
}
