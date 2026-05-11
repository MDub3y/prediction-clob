use crate::errors::ErrorCode;
use crate::logic::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub orderbook: AccountLoader<'info, Orderbook>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn handle_cancel_order(ctx: Context<CancelOrder>, order_idx: u32, side: u8) -> Result<()> {
    let mut ob = ctx.accounts.orderbook.load_mut()?;
    let order_side = if side == 0 {
        OrderSide::BID
    } else {
        OrderSide::ASK
    };

    let node = &ob.orders[order_idx as usize];
    require!(
        node.user == ctx.accounts.user.key(),
        ErrorCode::Unauthorized
    );
    require!(
        node.status.val == OrderStatus::OPEN.val,
        ErrorCode::OrderNotOpen
    );

    let remaining_qty = node.quantity.val - node.filled_quantity.val;
    let refund_amount = remaining_qty.checked_mul(node.price.val).unwrap();

    ctx.accounts.user_account.collateral_balance += refund_amount;

    unstitch_and_free(&mut ob, order_idx, order_side);

    msg!(
        "Order {} cancelled. {} collateral refunded to ledger.",
        order_idx,
        refund_amount
    );
    Ok(())
}
