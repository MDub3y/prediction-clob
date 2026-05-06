use crate::quantities::*;
use anchor_lang::prelude::*;

pub const MAX_ORDERS: usize = 1024;
pub const SENTINEL: u32 = u32::MAX;

#[zero_copy]
#[derive(Debug, PartialEq, Eq)]
pub struct OrderSide {
    pub val: u8,
}

impl OrderSide {
    pub const BID: Self = Self { val: 0 };
    pub const ASK: Self = Self { val: 1 };
}

#[zero_copy]
#[derive(Debug, PartialEq, Eq)]
pub struct OrderStatus {
    pub val: u8,
}

impl OrderStatus {
    pub const OPEN: Self = Self { val: 0 };
    pub const FILLED: Self = Self { val: 1 };
    pub const PARTIALLY_FILLED: Self = Self { val: 2 };
    pub const CANCELLED: Self = Self { val: 3 };
}

#[zero_copy]
#[derive(Debug, PartialEq)]
pub struct OrderNode {
    pub user: Pubkey,
    pub order_id: u64,
    pub price: Ticks,
    pub quantity: BaseLots,
    pub filled_quantity: BaseLots,
    pub timestamp: i64,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub padding: [u8; 6],
    pub next: u32,
    pub prev: u32,
}

#[account(zero_copy)]
pub struct Orderbook {
    pub market: Pubkey,
    pub outcome_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub bid_head: u32,
    pub ask_head: u32,
    pub free_head: u32,
    pub active_orders: u32,
    pub last_order_id: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
    pub orders: [OrderNode; MAX_ORDERS],
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
