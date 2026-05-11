use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod logic;
pub mod quantities;
pub mod state;

use instructions::*;

declare_id!("2cffJrXyZjoN1jT2BB671BjeqfxaJzHGgfjAd8QZQ8qh");

#[program]
pub mod prediction_clob {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_id: u32,
        deadline: i64,
        o_a: Pubkey,
        o_b: Pubkey,
        collateral_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize_market::handle_initialize_market(
            ctx,
            market_id,
            deadline,
            o_a,
            o_b,
            collateral_mint,
        )
    }

    pub fn initialize_user_account(ctx: Context<InitializeUserAccount>) -> Result<()> {
        instructions::initialize_user_account::handle_initialize_user_account(ctx)
    }

    pub fn initialize_orderbook(ctx: Context<InitializeOrderbook>) -> Result<()> {
        instructions::initialize_orderbook::handle_initialize_orderbook(ctx)
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        is_buying_a: bool,
        quantity: u64,
        price: u64,
    ) -> Result<()> {
        instructions::place_order::handle_place_order(ctx, is_buying_a, quantity, price)
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, order_idx: u32, side: u8) -> Result<()> {
        instructions::cancel_order::handle_cancel_order(ctx, order_idx, side)
    }

    pub fn split(ctx: Context<SplitMerge>, amount: u64) -> Result<()> {
        instructions::split_merge::handle_split(ctx, amount)
    }

    pub fn merge(ctx: Context<SplitMerge>, amount: u64) -> Result<()> {
        instructions::split_merge::handle_merge(ctx, amount)
    }

    pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
        instructions::sweep::handle_sweep_fees(ctx)
    }

    pub fn claim_collateral(ctx: Context<ClaimFunds>) -> Result<()> {
        instructions::claim_funds::handle_claim_collateral(ctx)
    }

    pub fn propose_outcome(ctx: Context<ProposeOutcome>, reported_outcome: u8) -> Result<()> {
        instructions::resolution::handle_propose_outcome(ctx, reported_outcome)
    }

    pub fn finalize_market(ctx: Context<FinalizeMarket>) -> Result<()> {
        instructions::resolution::handle_finalize_market(ctx)
    }
}
