use crate::logic::*;
use crate::quantities::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Transfer;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub orderbook_a: AccountLoader<'info, Orderbook>,
    #[account(mut)]
    pub orderbook_b: AccountLoader<'info, Orderbook>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub outcome_a_mint: Account<'info, Mint>,
    #[account(mut)]
    pub outcome_b_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_outcome_a_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_outcome_b_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_collateral_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handle_place_order(
    ctx: Context<PlaceOrder>,
    is_order_for_a: bool,
    quantity: u64,
    price: u64,
    side: u8,
    is_market: bool,
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let order_side = if side == 0 {
        OrderSide::BID
    } else {
        OrderSide::ASK
    };

    let target_ob_loader = if is_order_for_a {
        &ctx.accounts.orderbook_a
    } else {
        &ctx.accounts.orderbook_b
    };
    let mut ob = target_ob_loader.load_mut()?;

    if order_side == OrderSide::BID {
        let total_cost = quantity.checked_mul(price).unwrap() / 100;

        require!(
            user_account.collateral_balance >= total_cost,
            ErrorCode::InsufficientCollateral
        );

        user_account.collateral_balance -= total_cost;
        user_account.collateral_locked += total_cost;
    } else {
        let current_balance = if is_order_for_a {
            user_account.outcome_a_balance
        } else {
            user_account.outcome_b_balance
        };

        require!(current_balance >= quantity, ErrorCode::InsufficientShares);

        if is_order_for_a {
            user_account.outcome_a_balance -= quantity;
            user_account.outcome_a_locked += quantity;
        } else {
            user_account.outcome_b_balance -= quantity;
            user_account.outcome_b_locked += quantity;
        }
    }

    let match_result = execute_match(&mut ob, order_side, quantity, price, is_market);

    if match_result.base_filled > 0 {
        if order_side == OrderSide::BID {
            let filled_cost = match_result.quote_filled / 100;
            user_account.collateral_locked -= filled_cost;
            if is_order_for_a {
                user_account.outcome_a_balance += match_result.base_filled;
            } else {
                user_account.outcome_b_balance += match_result.base_filled;
            }
        } else {
            if is_order_for_a {
                user_account.outcome_a_locked -= match_result.base_filled;
            } else {
                user_account.outcome_b_locked -= match_result.base_filled;
            }
            user_account.collateral_balance += match_result.quote_filled / 100;
        }
    }

    if !is_market && match_result.remaining_quantity > 0 {
        if let Some(idx) = pop_free_node(&mut ob) {
            let node = &mut ob.orders[idx as usize];
            node.user = ctx.accounts.user.key();
            node.price = Ticks { val: price };
            node.quantity = BaseLots {
                val: match_result.remaining_quantity,
            };
            node.side = order_side;
            node.status = OrderStatus::OPEN;
            node.timestamp = ctx.accounts.clock.unix_timestamp;
            insert_sorted(&mut ob, idx, order_side);
        }
    } else if is_market && match_result.remaining_quantity > 0 {
        if order_side == OrderSide::BID {
            let unused_cost = match_result.remaining_quantity.checked_mul(price).unwrap() / 100;
            user_account.collateral_locked -= unused_cost;
            user_account.collateral_balance += unused_cost;
        } else {
            if is_order_for_a {
                user_account.outcome_a_locked -= match_result.remaining_quantity;
                user_account.outcome_a_balance += match_result.remaining_quantity;
            } else {
                user_account.outcome_b_locked -= match_result.remaining_quantity;
                user_account.outcome_b_balance += match_result.remaining_quantity;
            }
        }
    }

    Ok(())
}
