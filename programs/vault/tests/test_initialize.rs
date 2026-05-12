
use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, Space, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize() {
    let program_id = vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let (vault_state, _bump) =
        Pubkey::find_program_address(&[b"state", payer.pubkey().as_ref()], &program_id);

    let (vault, _vault_bumf) = Pubkey::find_program_address(&[b"vault", vault_state.as_ref()], &program_id);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &vault::instruction::Initialize {}.data(),
        vault::accounts::Initialize {
            user: payer.pubkey(),
            vault_state,
            vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "send_transaction failed: {:?}", res.err());

    let account = svm
        .get_account(&vault_state)
        .expect("vault_state account was not created");
    assert_eq!(account.owner, program_id, "vault_state owner is not the vault program");
    assert_eq!(
        account.data.len(),
        8 + vault::VaultState::INIT_SPACE,
        "vault_state data length does not match 8 + VaultState::INIT_SPACE"
    );
    assert!(account.lamports > 0, "vault_state has no lamports (not rent-funded)");
}
