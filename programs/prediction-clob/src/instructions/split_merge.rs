use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct SplitMerge<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub outcome_a_mint: Account<'info, Mint>,
    #[account(mut)]
    pub outcome_b_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_outcome_a_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_outcome_b_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_collateral_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handle_split(ctx: Context<SplitMerge>, amount: u64) -> Result<()> {
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_collateral_ata.to_account_info(),
                to: ctx.accounts.market_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount * 100,
    )?;

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"market", &market_id_bytes, &[ctx.accounts.market.bump]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.outcome_a_mint.to_account_info(),
                to: ctx.accounts.user_outcome_a_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.outcome_b_mint.to_account_info(),
                to: ctx.accounts.user_outcome_b_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;
    Ok(())
}

pub fn handle_merge(ctx: Context<SplitMerge>, amount: u64) -> Result<()> {
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.outcome_a_mint.to_account_info(),
                from: ctx.accounts.user_outcome_a_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.outcome_b_mint.to_account_info(),
                from: ctx.accounts.user_outcome_b_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"market", &market_id_bytes, &[ctx.accounts.market.bump]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.market_vault.to_account_info(),
                to: ctx.accounts.user_collateral_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[seeds],
        ),
        amount * 100,
    )?;
    Ok(())
}
