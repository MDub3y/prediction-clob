use crate::state::*;

pub struct MarketPriceInfo {
    pub bid: u64,
    pub ask: u64,
    pub midpoint: u64,
}

pub fn get_midpoint_price(ob: &Orderbook) -> MarketPriceInfo {
    let mut best_bid = 0u64;
    let mut best_ask = 100u64;

    if ob.bid_head != SENTINEL {
        best_bid = ob.orders[ob.bid_head as usize].price.val;
    }

    if ob.ask_head != SENTINEL {
        best_ask = ob.orders[ob.ask_head as usize].price.val;
    }

    let midpoint = if ob.bid_head != SENTINEL && ob.ask_head != SENTINEL {
        (best_bid + best_ask) / 2
    } else {
        ob.last_traded_price
    };

    MarketPriceInfo {
        bid: best_bid,
        ask: best_ask,
        midpoint,
    }
}
