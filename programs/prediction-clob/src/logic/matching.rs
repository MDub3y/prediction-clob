use anchor_lang::solana_program::pubkey::Pubkey;

use crate::logic::linked_list::*;
use crate::state::*;

pub struct MatchResult {
    pub base_filled: u64,
    pub quote_filled: u64,
    pub remaining_quantity: u64,
    pub makers_to_settle: Vec<FilledMaker>,
}

pub struct FilledMaker {
    pub user: Pubkey,
    pub base_delta: u64,
    pub quote_delta: u64,
}

pub fn execute_match(
    ob: &mut Orderbook,
    taker_side: OrderSide,
    taker_quantity: u64,
    limit_price: u64,
) -> MatchResult {
    let mut remaining = taker_quantity;
    let mut total_quote = 0u64;
    let mut total_base = 0u64;
    let mut makers = Vec::new();

    let mut maker_idx = if taker_side == OrderSide::BID {
        ob.ask_head
    } else {
        ob.bid_head
    };

    while maker_idx != SENTINEL && remaining > 0 {
        let (maker_user, maker_price, fill_amt, is_full_fill) = {
            let maker_node = &mut ob.orders[maker_idx as usize];

            let price_ok = if taker_side == OrderSide::BID {
                maker_node.price.val <= limit_price
            } else {
                maker_node.price.val >= limit_price
            };
            if !price_ok {
                break;
            }

            let available = maker_node.quantity.val - maker_node.filled_quantity.val;
            let fill = std::cmp::min(remaining, available);

            maker_node.filled_quantity.val += fill;
            let full = maker_node.filled_quantity.val == maker_node.quantity.val;

            (maker_node.user, maker_node.price.val, fill, full)
        };

        remaining -= fill_amt;
        total_base += fill_amt;
        total_quote += fill_amt * maker_price;

        makers.push(FilledMaker {
            user: maker_user,
            base_delta: fill_amt,
            quote_delta: fill_amt * maker_price,
        });

        if is_full_fill {
            let next_ptr = ob.orders[maker_idx as usize].next;
            unstitch_and_free(ob, maker_idx, taker_side.opposite());
            maker_idx = next_ptr;
        } else {
            break;
        }
    }

    MatchResult {
        base_filled: total_base,
        quote_filled: total_quote,
        remaining_quantity: remaining,
        makers_to_settle: makers,
    }
}
