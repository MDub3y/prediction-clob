use crate::logic::*;
use crate::quantities::*;
use crate::state::*;
use anchor_lang::prelude::*;
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
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_collateral_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handle_place_order(
    ctx: Context<PlaceOrder>,
    is_buying_a: bool,
    quantity: u64,
    price: u64,
    side: u8,
) -> Result<()> {
    let mut ob_a = ctx.accounts.orderbook_a.load_mut()?;
    let mut ob_b = ctx.accounts.orderbook_b.load_mut()?;
    let target_ob = if is_buying_a { &mut ob_a } else { &mut ob_b };

    let order_side = if side == 0 {
        OrderSide::BID
    } else {
        OrderSide::ASK
    };

    let match_result = execute_match(target_ob, order_side, quantity, price);

    if order_side == OrderSide::BID {
        let cost = quantity
            .checked_mul(price)
            .unwrap()
            .checked_div(100)
            .unwrap();
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_collateral_ata.to_account_info(),
                    to: ctx.accounts.market_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            cost,
        )?;
    } else {
        let target_ata = if is_buying_a {
            ctx.accounts.user_outcome_a_ata.to_account_info()
        } else {
            ctx.accounts.user_outcome_b_ata.to_account_info()
        };

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: target_ata,
                    to: ctx.accounts.market_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            quantity,
        )?;
    }

    if match_result.remaining_quantity > 0 {
        if let Some(idx) = pop_free_node(target_ob) {
            let node = &mut target_ob.orders[idx as usize];
            node.user = ctx.accounts.user.key();
            node.price = Ticks { val: price };
            node.quantity = BaseLots { val: quantity };
            node.side = order_side;
            node.status = OrderStatus::OPEN;
            node.timestamp = ctx.accounts.clock.unix_timestamp;
            insert_sorted(target_ob, idx, order_side);
        }
    }

    Ok(())
}
