use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Side { Bid, Ask }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderType { Limit, Market }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub order_id: u64,
    pub trader: Pubkey,
    pub side: Side,
    pub order_type: OrderType,
    pub price: u64,
    pub quantity: u64,
    pub filled_quantity: u64,
    pub timestamp: i64,
    pub status: OrderStatus,
}