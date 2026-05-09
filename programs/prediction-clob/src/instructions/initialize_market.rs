use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_market(
    ctx: Context<InitializeMarket>,
    market_id: u32,
    settlement_deadline: i64,
    outcome_a_mint: Pubkey,
    outcome_b_mint: Pubkey,
    collateral_mint: Pubkey,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    market.authority = ctx.accounts.authority.key();
    market.market_id = market_id;
    market.settlement_deadline = settlement_deadline;
    market.outcome_a_mint = outcome_a_mint;
    market.outcome_b_mint = outcome_b_mint;
    market.collateral_mint = collateral_mint;
    market.is_settled = false;
    market.bump = ctx.bumps.market;

    Ok(())
}
