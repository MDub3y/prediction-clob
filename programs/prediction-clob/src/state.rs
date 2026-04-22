use anchor_lang::prelude::*;
use std::collections::BTreeMap;
use crate::quantities::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderSide { Buy, Sell }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderStatus { Open, Filled, PartiallyFilled, Cancelled }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
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

#[account]
pub struct Orderbook {
    pub market: Pubkey,
    pub outcome_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub bids: BTreeMap<Ticks, Vec<Order>>,
    pub asks: BTreeMap<Ticks, Vec<Order>>, 
    pub last_order_id: u64,
    pub bump: u8,
}