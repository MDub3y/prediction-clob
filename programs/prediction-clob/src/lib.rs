use anchor_lang::prelude::*;

pub mod state;
pub mod quantities;
pub mod instructions;

use instructions::*;
use state::*;

declare_id!("2cffJrXyZjoN1jT2BB671BjeqfxaJzHGgfjAd8QZQ8qh");

#[program]
pub mod prediction_clob {
    use super::*;

    pub fn initialize_orderbook(ctx: Context<InitializeOrderbook>) -> Result<()> {
        let ob = &mut ctx.accounts.orderbook;
        
        ob.market = ctx.accounts.market.key();
        ob.outcome_mint = ctx.accounts.outcome_mint.key();
        
        ob.collateral_mint = ctx.accounts.market.collateral_mint;
        
        ob.bids = Vec::new();
        ob.asks = Vec::new();
        
        ob.last_order_id = 0;
        ob.bump = ctx.bumps.orderbook;
        
        msg!("Orderbook initialized for outcome: {:?}", ob.outcome_mint);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
