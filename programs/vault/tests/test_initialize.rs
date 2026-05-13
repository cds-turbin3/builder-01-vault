use {
    anchor_lang::{
        prelude::msg,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/vault.so");
    let _ = svm.add_program(program_id, bytes);
    svm.airdrop(&payer.pubkey(), 10_000_000_000)
        .expect("Airdrop failed");
    (svm, payer)
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let program_id = vault::id();
    let (mut svm, payer) = setup();

    let user = payer.pubkey();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", user.as_ref()], &program_id);
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &program_id);

    // Initialize
    let init_ix = Instruction {
        program_id,
        accounts: vault::accounts::Initialize {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: vault::instruction::Initialize {}.data(),
    };

    let message = Message::new(&[init_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx1 = svm.send_transaction(transaction).unwrap();

    msg!(" Initialize transaction successful");
    msg!(" Tx Signature {}", tx1.signature);

    let vault_state_account = svm.get_account(&vault_state_pda).unwrap();
    let vault_state =
        vault::state::VaultState::try_deserialize(&mut vault_state_account.data.as_ref()).unwrap();

    assert_eq!(vault_state.vault_bump, vault_bump);
    assert_eq!(vault_state.state_bump, state_bump);
    //
    // Deposit 1 Sol
    let deposit_amount: u64 = 1_000_000_000;
    let deposit_ix = Instruction {
        program_id,
        accounts: vault::accounts::Deposit {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: vault::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
    };

    let message = Message::new(&[deposit_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx2 = svm.send_transaction(transaction).unwrap();

    msg!(" Deposit transaction successful");
    msg!(" Tx Signature {}", tx2.signature);

    let vault_balance_after_deposit = svm.get_balance(&vault_pda).unwrap();
    assert_eq!(vault_balance_after_deposit, deposit_amount);
    msg!("Balance after deposit: {}", vault_balance_after_deposit);

    // Withdraw
    let withdraw_amount: u64 = 500_000_000;
    let withdraw_ix = Instruction {
        program_id,
        accounts: vault::accounts::Withdraw {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: vault::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
    };

    let message = Message::new(&[withdraw_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx3 = svm.send_transaction(transaction).unwrap();

    msg!(" Withdraw transaction successful");
    msg!(" Tx Signature {}", tx3.signature);

    let vault_balance_after_withdraw = svm.get_balance(&vault_pda).unwrap();
    assert_eq!(vault_balance_after_withdraw, withdraw_amount);
    msg!("Balance after deposit: {}", vault_balance_after_withdraw);

    // Close
    let close_amount = svm.get_balance(&vault_pda).unwrap();
    let close_ix = Instruction {
        program_id,
        accounts: vault::accounts::Close {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: vault::instruction::Close {}.data(),
    };

    let message = Message::new(&[close_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx4 = svm.send_transaction(transaction).unwrap();

    msg!(" Close transaction successful");
    msg!(" Tx Signature {}", tx4.signature);

    assert!(svm.get_account(&vault_pda).is_none());
    assert!(svm.get_account(&vault_state_pda).is_none());

    let user_balance_after_close = svm.get_balance(&user).unwrap();
    assert!(user_balance_after_close > close_amount); // oh boy, bad episilon
}
