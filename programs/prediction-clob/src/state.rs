use crate::quantities::*;
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

pub const MAX_ORDERS: usize = 1000;
pub const SENTINEL: u32 = u32::MAX;

#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct OrderSide(pub u8);

impl OrderSide {
    pub const BID: Self = Self(0);
    pub const ASK: Self = Self(1);
}

#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct OrderStatus {
    pub val: u8,
}

impl OrderStatus {
    pub const OPEN: Self = Self { val: 0 };
    pub const FILLED: Self = Self { val: 1 };
    pub const PARTIALLY_FILLED: Self = Self { val: 2 };
    pub const CANCELLED: Self = Self { val: 3 };
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, InitSpace, Pod, Zeroable)]
pub struct Order {
    pub user: Pubkey,
    pub order_id: u64,
    pub price: Ticks,
    pub quantity: BaseLots,
    pub filled_quantity: BaseLots,
    pub timestamp: i64,
    pub side: OrderSide,
    pub status: OrderStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq, InitSpace)]
pub struct PriceLevel {
    pub price: Ticks,
    #[max_len(10)]
    pub orders: Vec<Order>,
}

#[account]
#[derive(InitSpace)]
pub struct Orderbook {
    pub market: Pubkey,
    pub outcome_mint: Pubkey,
    pub collateral_mint: Pubkey,

    #[max_len(50)]
    pub bids: Vec<PriceLevel>,
    #[max_len(50)]
    pub asks: Vec<PriceLevel>,

    pub last_order_id: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub authority: Pubkey,
    pub market_id: u32,
    pub settlement_deadline: i64,
    pub outcome_a_mint: Pubkey,
    pub outcome_b_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub collateral_vault: Pubkey,
    pub is_settled: bool,
    pub winning_outcome: Option<u8>,
    pub total_collateral_locked: u64,
    pub bump: u8,
}
