use crate::state::*;

pub fn pop_free_node(ob: &mut Orderbook) -> Option<u32> {
    if ob.free_head == SENTINEL {
        return None;
    }
    let index = ob.free_head;
    ob.free_head = ob.orders[index as usize].next;
    ob.active_orders += 1;
    Some(index)
}

pub fn push_free_node(ob: &mut Orderbook, index: u32) {
    ob.orders[index as usize].status = OrderStatus::CANCELLED;
    ob.orders[index as usize].next = ob.free_head;
    ob.orders[index as usize].prev = SENTINEL;
    ob.free_head = index;
    ob.active_orders -= 1;
}

pub fn insert_sorted(ob: &mut Orderbook, new_index: u32, side: OrderSide) {
    let mut curr_idx = if side == OrderSide::BID {
        ob.bid_head
    } else {
        ob.ask_head
    };
    let mut prev_idx = SENTINEL;

    let new_price = ob.orders[new_index as usize].price.val;

    while curr_idx != SENTINEL {
        let curr_node = &ob.orders[curr_idx as usize];

        let should_insert_before = if side == OrderSide::BID {
            new_price > curr_node.price.val
        } else {
            new_price < curr_node.price.val
        };

        if should_insert_before {
            break;
        }

        prev_idx = curr_idx;
        curr_idx = curr_node.next;
    }

    ob.orders[new_index as usize].next = curr_idx;
    ob.orders[new_index as usize].prev = prev_idx;

    if prev_idx == SENTINEL {
        if side == OrderSide::BID {
            ob.bid_head = new_index;
        } else {
            ob.ask_head = new_index;
        }
    } else {
        ob.orders[prev_idx as usize].next = new_index;
    }

    if curr_idx != SENTINEL {
        ob.orders[curr_idx as usize].prev = new_index;
    }
}
