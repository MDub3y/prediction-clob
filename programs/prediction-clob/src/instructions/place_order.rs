use crate::logic::*;
use crate::quantities::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

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
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_collateral_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_place_order(
    ctx: Context<PlaceOrder>,
    is_buying_a: bool,
    quantity: u64,
    price: u64,
) -> Result<()> {
    let mut ob_a = ctx.accounts.orderbook_a.load_mut()?;
    let mut ob_b = ctx.accounts.orderbook_b.load_mut()?;

    let complement_price = 100u64.saturating_sub(price);

    let target_ob = if is_buying_a { &mut ob_a } else { &mut ob_b };
    let match_result = execute_match(target_ob, OrderSide::BID, quantity, price);

    let cost = quantity.checked_mul(price).unwrap();
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_collateral_ata.to_account_info(),
                to: ctx.accounts.market_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        cost,
    )?;

    let user_ledger = &mut ctx.accounts.user_account;
    if is_buying_a {
        user_ledger.outcome_a_balance += match_result.base_filled;
    } else {
        user_ledger.outcome_b_balance += match_result.base_filled;
    }

    if match_result.remaining_quantity > 0 {
        if let Some(idx) = pop_free_node(target_ob) {
            let node = &mut target_ob.orders[idx as usize];
            node.user = ctx.accounts.user.key();
            node.price = Ticks { val: price };
            node.quantity = BaseLots { val: quantity };
            node.filled_quantity = BaseLots {
                val: match_result.base_filled,
            };
            node.side = OrderSide::BID;
            node.status = OrderStatus::OPEN;
            insert_sorted(target_ob, idx, OrderSide::BID);
        }
    }

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
    #[msg("No funds available to claim.")]
    NothingToClaim,
}
