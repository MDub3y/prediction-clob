use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeUserAccount<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + UserAccount::INIT_SPACE,
        seeds = [b"user", market.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    pub market: Account<'info, Market>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_user_account(ctx: Context<InitializeUserAccount>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    user_account.owner = ctx.accounts.owner.key();
    user_account.market = ctx.accounts.market.key();
    user_account.collateral_balance = 0;
    user_account.outcome_a_balance = 0;
    user_account.outcome_b_balance = 0;
    user_account.bump = ctx.bumps.user_account;
    Ok(())
}
