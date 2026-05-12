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

pub fn handle_initialize_orderbook(
    ctx: Context<InitializeOrderbook>,
    start_index: u32,
    batch_size: u32,
) -> Result<()> {
    let mut ob = if start_index == 0 {
        ctx.accounts.orderbook.load_init()?
    } else {
        ctx.accounts.orderbook.load_mut()?
    };

    if start_index == 0 {
        ob.market = ctx.accounts.market.key();
        ob.outcome_mint = ctx.accounts.outcome_mint.key();
        ob.collateral_mint = ctx.accounts.market.collateral_mint;
        ob.bid_head = SENTINEL;
        ob.ask_head = SENTINEL;
        ob.free_head = 0;
        ob.active_orders = 0;
        ob.last_traded_price = 0;
    }

    let end_index = std::cmp::min(start_index + batch_size, MAX_ORDERS as u32);

    for i in start_index..end_index {
        let node = &mut ob.orders[i as usize];
        node.next = if i == (MAX_ORDERS - 1) as u32 {
            SENTINEL
        } else {
            i + 1
        };
        node.prev = SENTINEL;
        node.status = OrderStatus::CANCELLED;
    }

    msg!("Initialized nodes {} to {}", start_index, end_index - 1);
    Ok(())
}
