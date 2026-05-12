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

    pub collateral_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = collateral_mint,
        token::authority = market,
        seeds = [b"vault", market.key().as_ref(), collateral_mint.key().as_ref()],
        bump
    )]
    pub market_vault: Account<'info, anchor_spl::token::TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_initialize_market(
    ctx: Context<InitializeMarket>,
    market_id: u32,
    settlement_deadline: i64,
    outcome_a_mint: Pubkey,
    outcome_b_mint: Pubkey,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    market.authority = ctx.accounts.authority.key();
    market.market_id = market_id;
    market.settlement_deadline = settlement_deadline;
    market.outcome_a_mint = outcome_a_mint;
    market.outcome_b_mint = outcome_b_mint;
    market.collateral_mint = ctx.accounts.collateral_mint.key();
    market.is_settled = false;
    market.bump = ctx.bumps.market;

    Ok(())
}
