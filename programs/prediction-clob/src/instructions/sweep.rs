use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)]
    pub orderbook: AccountLoader<'info, Orderbook>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub platform_fee_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub authority: Signer<'info>,
}

pub fn handle_sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
    let mut ob = ctx.accounts.orderbook.load_mut()?;
    let amount = ob.unclaimed_fees;

    require!(amount > 0, ErrorCode::NothingToClaim);
    require!(
        ctx.accounts.authority.key() == ctx.accounts.market.authority,
        ErrorCode::Unauthorized
    );

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"market", &market_id_bytes, &[ctx.accounts.market.bump]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.market_vault.to_account_info(),
                to: ctx.accounts.platform_fee_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    ob.unclaimed_fees = 0;
    Ok(())
}
