use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
#[instruction(market: Market)]
pub struct InitializeOrderbook<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<Orderbook>(),
        seeds = [b"orderbook", market.key().as_ref(), outcome_mint.key().as_ref()],
        bump
    )]
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
        ob.orders[i].prev = SENTINEL;
        ob.orders[i].status = OrderStatus::CANCELLED;
    }

    msg!("Orderbook initialized for outcome: {:?}", ob.outcome_mint);
    Ok(())
}
