use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ProposeOutcome<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub reporter: Signer<'info>,
}

pub fn handle_propose_outcome(ctx: Context<ProposeOutcome>, reported_outcome: u8) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    require!(!market.is_settled, ErrorCode::MarketAlreadySettled);
    require!(
        clock.unix_timestamp > market.settlement_deadline,
        ErrorCode::MarketNotExpired
    );

    // Set the proposed outcome and the challenge clock
    market.reported_outcome = Some(reported_outcome);
    market.report_timestamp = clock.unix_timestamp;
    // 24-hour challenge period (86400 seconds)
    market.challenge_end_timestamp = clock.unix_timestamp + 86400;

    msg!(
        "Outcome proposed: {}. Challenge window ends in 24h.",
        reported_outcome
    );
    Ok(())
}

#[derive(Accounts)]
pub struct FinalizeMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
}

pub fn handle_finalize_market(ctx: Context<FinalizeMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let clock = Clock::get()?;

    let reported = market.reported_outcome.ok_or(ErrorCode::NothingToClaim)?; // Reuse or add error
    require!(
        clock.unix_timestamp > market.challenge_end_timestamp,
        ErrorCode::ChallengePeriodActive
    );

    market.winning_outcome = Some(reported);
    market.is_settled = true;

    msg!("Market finalized. Winner: {}", reported);
    Ok(())
}
