use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

// smallest unit of price movement
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, InitSpace, Pod, Zeroable)]
pub struct Ticks(pub u64);

// smallest unit of outcome token
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, InitSpace, Pod, Zeroable)]
pub struct BaseLots(pub u64);

// smallest unit of collateral
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, InitSpace, Pod, Zeroable)]
pub struct QuoteLots(pub u64);
