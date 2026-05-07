use crate::logic::*;
use crate::quantities::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub orderbook: AccountLoader<'info, Orderbook>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = market_vault.key() == market.collateral_vault
    )]
    pub market_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_place_order(
    ctx: Context<PlaceOrder>,
    side: u8,
    quantity: u64,
    price: u64,
    is_market: bool,
) -> Result<()> {
    let mut ob = ctx.accounts.orderbook.load_mut()?;
    let taker_side = OrderSide { val: side };
    let timestamp = ctx.accounts.clock.unix_timestamp;

    // pre-transfer of funds to vault
    // for BID: taker sends base asset (quantity * price)
    // for ASK: taker sends outcome token (quantity)
    let deposit_amount = if taker_side == OrderSide::BID {
        quantity.checked_mul(price).ok_or(ErrorCode::MathOverflow)?
    } else {
        quantity
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.market_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        deposit_amount,
    )?;

    let match_result = execute_match(&mut ob, taker_side, quantity, price);

    if !is_market && match_result.remaining_quantity > 0 {
        if let Some(new_idx) = pop_free_node(&mut ob) {
            let node = &mut ob.orders[new_idx as usize];
            node.user = ctx.accounts.user.key();
            node.price = Ticks { val: price };
            node.quantity = BaseLots { val: quantity };
            node.filled_quantity = BaseLots {
                val: match_result.base_filled,
            };
            node.side = taker_side;
            node.status = OrderStatus::OPEN;
            node.timestamp = timestamp;

            insert_sorted(&mut ob, new_idx, taker_side);
        } else {
            return Err(error!(ErrorCode::BookFull));
        }
    } else if is_market && match_result.remaining_quantity > 0 {
        return Err(error!(ErrorCode::SlippageExceeded));
    }

    let market = &mut ctx.accounts.market;
    market.total_collateral_locked = market
        .total_collateral_locked
        .checked_add(match_result.quote_filled)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!(
        "Order processed. Filled: {} base, {} quote. Remaining: {}",
        match_result.base_filled,
        match_result.quote_filled,
        match_result.remaining_quantity
    );

    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("The orderbook is full.")]
    BookFull,
    #[msg("Market order could not be filled within slippage limits.")]
    SlippageExceeded,
    #[msg("Math overflow.")]
    MathOverflow,
}
