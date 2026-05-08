use crate::instructions::place_order::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct ClaimFunds<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_collateral_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub user: Signer<'info>,
}

pub fn handle_claim_collateral(ctx: Context<ClaimFunds>) -> Result<()> {
    let seat = &mut ctx.accounts.user_account;
    let amount = seat.collateral_balance;
    require!(amount > 0, ErrorCode::NothingToClaim);

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"market", &market_id_bytes, &[ctx.accounts.market.bump]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_collateral_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    seat.collateral_balance = 0;
    Ok(())
}
