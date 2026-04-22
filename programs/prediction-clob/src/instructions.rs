use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;

#[derive(Accounts)]
#[instruction(market: Market)]
pub struct InitializeOrderbook<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Orderbook::INIT_SPACE,
        seeds = [b"orderbook", market.key().as_ref(), outcome_mint.key().as_ref()],
        bump
    )]
    pub orderbook: Account<'info, Orderbook>,

    pub market: Account<'info, Market>,
    pub outcome_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}