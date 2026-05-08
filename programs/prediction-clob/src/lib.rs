use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod logic;
pub mod quantities;
pub mod state;

use instructions::*;

declare_id!("2cffJrXyZjoN1jT2BB671BjeqfxaJzHGgfjAd8QZQ8qh");

#[program]
pub mod prediction_clob {
    use super::*;

    pub fn initialize_orderbook(ctx: Context<InitializeOrderbook>) -> Result<()> {
        instructions::initialize_orderbook::handle_initialize_orderbook(ctx)
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        is_buying_a: bool,
        quantity: u64,
        price: u64,
    ) -> Result<()> {
        instructions::place_order::handle_place_order(ctx, is_buying_a, quantity, price)
    }
}
