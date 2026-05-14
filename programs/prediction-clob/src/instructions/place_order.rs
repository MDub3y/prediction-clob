use crate::logic::*;
use crate::quantities::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::MintTo;
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
) -> Result<()> {
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
        let cost = quantity
            .checked_mul(price)
            .unwrap()
            .checked_div(100)
            .unwrap();
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
    } else {
        let target_ata = if is_order_for_a {
            ctx.accounts.user_outcome_a_ata.to_account_info()
        } else {
            ctx.accounts.user_outcome_b_ata.to_account_info()
        };

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: target_ata,
                    to: ctx.accounts.market_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            quantity,
        )?;
    }

    let match_result = execute_match(&mut ob, order_side, quantity, price);

    if match_result.base_filled > 0 {
        let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
        let seeds: &[&[u8]] = &[b"market", &market_id_bytes, &[ctx.accounts.market.bump]];

        if order_side == OrderSide::BID {
            let mint_to_info = if is_order_for_a {
                ctx.accounts.outcome_a_mint.to_account_info()
            } else {
                ctx.accounts.outcome_b_mint.to_account_info()
            };
            let ata_to_info = if is_order_for_a {
                ctx.accounts.user_outcome_a_ata.to_account_info()
            } else {
                ctx.accounts.user_outcome_b_ata.to_account_info()
            };

            token::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: mint_to_info,
                        to: ata_to_info,
                        authority: ctx.accounts.market.to_account_info(),
                    },
                    &[seeds],
                ),
                match_result.base_filled,
            )?;
        }
    }

    if match_result.remaining_quantity > 0 {
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
    }

    Ok(())
}
