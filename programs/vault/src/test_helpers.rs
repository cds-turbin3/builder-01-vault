//! Off-chain helpers for constructing vault instructions in tests.
//!
//! Every vault instruction's Accounts struct shares the same four-field shape:
//! `user`, `vault_state`, `vault`, `system_program`. This module bundles the
//! three pubkeys that actually vary into [`VaultAccs`] and provides:
//!
//! 1. `From<VaultAccs>` for each of the four `accounts::*` client mirrors, so
//!    callers can build any of them from one bundle.
//! 2. The [`VaultIx`] trait, which associates each `instruction::Variant`
//!    args struct with its companion `accounts::Variant` struct. A generic
//!    helper bounded on `VaultIx` can then build any vault instruction
//!    without macro dispatch, keeping the call site fully analyzable by
//!    rust-analyzer (hover, goto, autocomplete all work on the args struct).
//!
//! Gated `#[cfg(not(target_os = "solana"))]` so none of this lands in the
//! on-chain binary.

use crate::{accounts, instruction};
use anchor_lang::{prelude::Pubkey, solana_program::system_program};

#[derive(Copy, Clone, Debug)]
pub struct VaultAccs {
    pub user: Pubkey,
    pub state: Pubkey,
    pub vault: Pubkey,
}

macro_rules! impl_from_vault_accs {
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
        )*
    };
}

impl_from_vault_accs!(Initialize, Deposit, Withdraw, Close);

/// Associates a vault instruction's args struct with its accounts struct.
///
/// Implemented for each `instruction::Variant`, with `Accounts = accounts::Variant`.
/// Generic helpers (see `tests/test_initialize.rs::build_ix`) use this to
/// build any vault ix from `(VaultAccs, args)` without macro-based dispatch.
pub trait VaultIx: anchor_lang::InstructionData {
    type Accounts: From<VaultAccs> + anchor_lang::ToAccountMetas;
}

macro_rules! impl_vault_ix {
    ($($variant:ident),* $(,)?) => {
        $(
            impl VaultIx for instruction::$variant {
                type Accounts = accounts::$variant;
            }
        )*
    };
}

impl_vault_ix!(Initialize, Deposit, Withdraw, Close);
