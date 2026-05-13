use {
    anchor_lang::solana_program::system_program,
    anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers},
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    vault::state::VaultState,
};

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

    // Initialize
    let ix = ctx
        .program()
        .accounts(vault::accounts::Initialize {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        })
        .args(vault::instruction::Initialize {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&payer])
        .unwrap()
        .assert_success();

    let state: VaultState = ctx.get_account(&vault_state_pda).unwrap();
    assert_eq!(state.vault_bump, vault_bump);
    assert_eq!(state.state_bump, state_bump);

    // Deposit 1 SOL
    let deposit_amount: u64 = 1_000_000_000;
    let ix = ctx
        .program()
        .accounts(vault::accounts::Deposit {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        })
        .args(vault::instruction::Deposit {
            amount: deposit_amount,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&payer])
        .unwrap()
        .assert_success();
    ctx.svm.assert_sol_balance(&vault_pda, deposit_amount);

    // Withdraw half
    let withdraw_amount: u64 = 500_000_000;
    let ix = ctx
        .program()
        .accounts(vault::accounts::Withdraw {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        })
        .args(vault::instruction::Withdraw {
            amount: withdraw_amount,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&payer])
        .unwrap()
        .assert_success();
    ctx.svm
        .assert_sol_balance(&vault_pda, deposit_amount - withdraw_amount);

    // Close
    let user_balance_before_close = ctx.svm.get_balance(&user).unwrap();
    let ix = ctx
        .program()
        .accounts(vault::accounts::Close {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        })
        .args(vault::instruction::Close {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&payer])
        .unwrap()
        .assert_success();

    ctx.svm.assert_account_closed(&vault_pda);
    ctx.svm.assert_account_closed(&vault_state_pda);
    assert!(ctx.svm.get_balance(&user).unwrap() > user_balance_before_close);
}
