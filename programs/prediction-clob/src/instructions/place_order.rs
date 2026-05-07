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
    pub user_account: Account<'info, UserAccount>, // Taker's internal ledger
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = user_token_account.owner == user.key())]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = market_vault.key() == market.collateral_vault)]
    pub market_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
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

    // Pre-transfer: Escrow the maximum possible cost of this order
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

    // Execute the Match
    let match_result = execute_match(&mut ob, taker_side, quantity, price);

    // Post-Match: Update Taker's Internal Ledger
    // If I bid and matched, I spent USDC and gained Tokens
    let taker_ledger = &mut ctx.accounts.user_account;
    if taker_side == OrderSide::BID {
        // We Gain Outcome Tokens (Base)
        taker_ledger.outcome_a_balance = taker_ledger
            .outcome_a_balance
            .checked_add(match_result.base_filled)
            .ok_or(ErrorCode::MathOverflow)?;

        // If it was a Limit order, the remaining USDC stays "locked" in the OrderNode
        // If it was a Market order, we might need to refund excess USDC (simplified here)
    } else {
        // We Gain USDC (Quote)
        taker_ledger.collateral_balance = taker_ledger
            .collateral_balance
            .checked_add(match_result.quote_filled)
            .ok_or(ErrorCode::MathOverflow)?;
    }

    // 4. If Limit Order & remains: Add to Book
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
