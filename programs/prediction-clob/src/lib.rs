use anchor_lang::prelude::*;

declare_id!("2cffJrXyZjoN1jT2BB671BjeqfxaJzHGgfjAd8QZQ8qh");

#[program]
pub mod prediction_clob {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
