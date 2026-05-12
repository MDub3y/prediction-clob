use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct InitializeOrderbook<'info> {
    #[account(zero)]
    pub orderbook: AccountLoader<'info, Orderbook>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        constraint = outcome_mint.key() == market.outcome_a_mint || outcome_mint.key() == market.outcome_b_mint
    )]
    pub outcome_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_orderbook(ctx: Context<InitializeOrderbook>) -> Result<()> {
    let mut ob = ctx.accounts.orderbook.load_mut()?;

    ob.market = ctx.accounts.market.key();
    ob.outcome_mint = ctx.accounts.outcome_mint.key();
    ob.collateral_mint = ctx.accounts.market.collateral_mint;

    ob.bid_head = SENTINEL;
    ob.ask_head = SENTINEL;
    ob.free_head = 0;
    ob.active_orders = 0;
    ob.last_traded_price = 0;

    for i in 0..MAX_ORDERS {
        let node = &mut ob.orders[i];
        node.next = if i == MAX_ORDERS - 1 {
            SENTINEL
        } else {
            (i + 1) as u32
        };
        node.prev = SENTINEL;
        node.status = OrderStatus::CANCELLED;
    }

    msg!("Orderbook initialized for outcome: {:?}", ob.outcome_mint);

    Ok(())
}
