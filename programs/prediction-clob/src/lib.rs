use anchor_lang::prelude::*;

pub mod instructions;
pub mod quantities;
pub mod state;

use instructions::*;
use state::*;

declare_id!("2cffJrXyZjoN1jT2BB671BjeqfxaJzHGgfjAd8QZQ8qh");

#[program]
pub mod prediction_clob {
    use super::*;

    pub fn initialize_orderbook(ctx: Context<InitializeOrderbook>) -> Result<()> {
        let mut ob = ctx.accounts.orderbook.load_init()?;

        ob.market = ctx.accounts.market.key();
        ob.outcome_mint = ctx.accounts.outcome_mint.key();
        ob.collateral_mint = ctx.accounts.market.collateral_mint;

        ob.bid_head = SENTINEL;
        ob.ask_head = SENTINEL;
        ob.free_head = 0;
        ob.active_orders = 0;
        ob.last_order_id = 0;
        ob.bump = ctx.bumps.orderbook;

        for i in 0..MAX_ORDERS {
            ob.orders[i].next = if i == MAX_ORDERS - 1 {
                SENTINEL
            } else {
                (i + 1) as u32
            };
            ob.orders[i].status = OrderStatus::CANCELLED;
        }

        msg!("Orderbook initialized for outcome: {:?}", ob.outcome_mint);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
