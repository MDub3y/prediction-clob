use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
#[instruction(market: Market)]
pub struct InitializeOrderbook<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<Orderbook>(),
        seeds = [b"orderbook", market.key().as_ref(), outcome_mint.key().as_ref()],
        bump
    )]
    pub orderbook: AccountLoader<'info, Orderbook>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        constraint = outcome_mint.key() == market.outcome_a_mint || outcome_mint.key() == market.outcome_b_mint
    )]
    pub outcome_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
