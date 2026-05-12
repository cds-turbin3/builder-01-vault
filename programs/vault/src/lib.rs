pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BGDwZwiCdyRySqdwCnRhAUe1yzZ8jc2Rccde8wYZVQqN");

#[program]
pub mod vault {
    use super::*;

    // Initialize program and accounts
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    // depositing funds to that vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    // withdraw funds
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    // close vault
    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}
