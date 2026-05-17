use {
    anchor_lang::prelude::msg,
    anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers, TransactionResult},
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    vault::{instruction as vix, state::VaultState, test_helpers::VaultAccs},
};

// test helper
pub fn pretty_log(tx: &TransactionResult, test_name: &str) {
    msg!("\n\n### {}\n", test_name);
    msg!("\n```console");
    tx.print_logs_structured();
    msg!("\n```\n");
}

fn setup() -> (AnchorContext, Keypair) {
    let ctx = AnchorLiteSVM::build_with_program(
        vault::id(),
        include_bytes!("../../../target/deploy/vault.so"),
    );
    let payer = ctx.payer().insecure_clone();
    (ctx, payer)
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let (mut ctx, payer) = setup();
    let program_id = vault::id();
    let user = payer.pubkey();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", user.as_ref()], &program_id);
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &program_id);

    let accs = VaultAccs {
        user,
        state: vault_state_pda,
        vault: vault_pda,
    };

    // Initialize
    let ix = ctx.program().build_ix(accs, vix::Initialize {});
    let result = ctx.execute_instruction(ix, &[&payer]).unwrap();
    pretty_log(&result, "Vault Initialize");
    result.assert_success();

    let state: VaultState = ctx.get_account(&vault_state_pda).unwrap();
    assert_eq!(state.vault_bump, vault_bump);
    assert_eq!(state.state_bump, state_bump);

    // Deposit 1 SOL
    let deposit_amount: u64 = 1_000_000_000;
    let ix = ctx.program().build_ix(
        accs,
        vix::Deposit {
            amount: deposit_amount,
        },
    );
    let result = ctx.execute_instruction(ix, &[&payer]).unwrap();
    pretty_log(&result, "Vault Deposit");
    result.assert_success();
    ctx.svm.assert_sol_balance(&vault_pda, deposit_amount);

    // Withdraw half
    let withdraw_amount: u64 = 500_000_000;
    let ix = ctx.program().build_ix(
        accs,
        vix::Withdraw {
            amount: withdraw_amount,
        },
    );
    let result = ctx.execute_instruction(ix, &[&payer]).unwrap();
    pretty_log(&result, "Vault Withdraw");
    result.assert_success();
    ctx.svm
        .assert_sol_balance(&vault_pda, deposit_amount - withdraw_amount);

    // Close
    let user_balance_before_close = ctx.svm.get_balance(&user).unwrap();
    let ix = ctx.program().build_ix(accs, vix::Close {});
    let result = ctx.execute_instruction(ix, &[&payer]).unwrap();
    pretty_log(&result, "Vault Close");
    result.assert_success();

    ctx.svm.assert_account_closed(&vault_pda);
    ctx.svm.assert_account_closed(&vault_state_pda);
    assert!(ctx.svm.get_balance(&user).unwrap() > user_balance_before_close);
}
