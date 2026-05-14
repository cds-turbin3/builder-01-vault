//! Off-chain helpers for constructing vault instructions in tests.
//!
//! Every vault instruction's Accounts struct shares the same four-field shape:
//! `user`, `vault_state`, `vault`, `system_program`. This module bundles the
//! three pubkeys that actually vary into [`VaultAccs`] and provides:
//!
//! 1. `From<VaultAccs>` for each of the four `accounts::*` client mirrors.
//! 2. `BuildableIx<VaultAccs>` for each `instruction::*` args struct,
//!    pinning it to the matching accounts struct.
//!
//! With those impls in place, `ctx.program().build_ix(accs, args)` constructs
//! any vault ix in one call, and `build_ix_with(..., |a| ...)` covers the
//! negative-path case where you want a deliberately-wrong account.
//!
//! Gated `#[cfg(not(target_os = "solana"))]` so none of this lands in the
//! on-chain binary.

use crate::{accounts, instruction};
use anchor_lang::{prelude::Pubkey, solana_program::system_program};
use anchor_litesvm::BuildableIx;

#[derive(Copy, Clone, Debug)]
pub struct VaultAccs {
    pub user: Pubkey,
    pub state: Pubkey,
    pub vault: Pubkey,
}

macro_rules! impl_vault_bundle {
    ($($variant:ident),* $(,)?) => {
        $(
            impl From<VaultAccs> for accounts::$variant {
                fn from(a: VaultAccs) -> Self {
                    Self {
                        user: a.user,
                        vault_state: a.state,
                        vault: a.vault,
                        system_program: system_program::ID,
                    }
                }
            }

            impl BuildableIx<VaultAccs> for instruction::$variant {
                type Accounts = accounts::$variant;
            }
        )*
    };
}

impl_vault_bundle!(Initialize, Deposit, Withdraw, Close);
