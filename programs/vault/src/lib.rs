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

    // withdraw funds

    // close vault
}
