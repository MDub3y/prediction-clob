use crate::logic::linked_list::*;
use crate::quantities::*;
use crate::state::*;

pub struct MatchResult {
    pub base_filled: u64,
    pub quote_filled: u64,
    pub remaining_quantity: u64,
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

    let mut maker_idx = if taker_side == OrderSide::BID {
        ob.ask_head
    } else {
        ob.bid_head
    };

    while maker_idx != SENTINEL && remaining > 0 {
        let maker_node = &mut ob.orders[maker_idx as usize];

        let price_ok = if taker_side == OrderSide::BID {
            maker_node.price.val <= limit_price
        } else {
            maker_node.price.val >= limit_price
        };

        if !price_ok {
            break;
        }

        let maker_available = maker_node.quantity.val - maker_node.filled_quantity.val;
        let fill_amt = std::cmp::min(remaining, maker_available);

        maker_node.filled_quantity.val += fill_amt;
        remaining -= fill_amt;
        total_base += fill_amt;
        total_quote += fill_amt * maker_node.price.val;

        let next_maker = maker_node.next;

        if maker_node.filled_quantity.val == maker_node.quantity.val {
            maker_node.status = OrderStatus::FILLED;
            let p = maker_node.prev;
            let n = maker_node.next;
            if p != SENTINEL {
                ob.orders[p as usize].next = n;
            } else {
                if taker_side == OrderSide::BID {
                    ob.ask_head = n;
                } else {
                    ob.bid_head = n;
                }
            }
            if n != SENTINEL {
                ob.orders[n as usize].prev = p;
            }

            push_free_node(ob, maker_idx);
        }

        maker_idx = next_maker;
    }

    MatchResult {
        base_filled: total_base,
        quote_filled: total_quote,
        remaining_quantity: remaining,
    }
}
