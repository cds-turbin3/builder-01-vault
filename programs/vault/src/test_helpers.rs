//! Off-chain helpers for constructing vault instructions in tests.
//!
//! Every vault instruction's Accounts struct shares the same four-field shape:
//! `user`, `vault_state`, `vault`, `system_program`. This module bundles the
//! three pubkeys that actually vary into [`VaultAccs`] and provides
//! `From<VaultAccs>` for each of the four `accounts::*` client mirrors, so
//! callers can build any of them from one bundle.
//!
//! The [`vault_ix!`] macro folds the bundle + variant + args into a single
//! call.
//!
//! Gated `#[cfg(not(target_os = "solana"))]` so none of this lands in the
//! on-chain binary.

use crate::accounts;
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

/// Build an instruction for one of the vault's four entry points.
///
/// ```ignore
/// let accs = VaultAccs { user, state: vault_state_pda, vault: vault_pda };
/// let ix = vault_ix!(ctx, accs, Initialize);
/// let ix = vault_ix!(ctx, accs, Deposit, amount: 1_000_000_000);
/// ```
#[macro_export]
macro_rules! vault_ix {
    ($ctx:expr, $accs:expr, $variant:ident $(, $arg:ident : $val:expr)* $(,)?) => {
        $ctx.program()
            .accounts(<$crate::accounts::$variant>::from($accs))
            .args($crate::instruction::$variant { $($arg: $val),* })
            .instruction()
            .unwrap()
    };
}
