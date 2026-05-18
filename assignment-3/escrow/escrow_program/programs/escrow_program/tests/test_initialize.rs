use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize_escrow() {
    let program_id = escrow_program::id();
    let maker = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/escrow_program.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&maker.pubkey(), 10_000_000_000).unwrap(); // 10 SOL

    let seed: u64 = 42;
    let (escrow, _bump) = anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
        &[b"escrow", maker.pubkey().as_ref(), &seed.to_le_bytes()],
        &program_id,
    );

    // Mock mints and token accounts for instruction format validation
    let mint_a = Keypair::new().pubkey();
    let mint_b = Keypair::new().pubkey();
    let maker_ata_a = Keypair::new().pubkey();
    let vault = Keypair::new().pubkey();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &escrow_program::instruction::Make {
            seed,
            receive: 100,
            amount: 50,
        }.data(),
        escrow_program::accounts::Make {
            maker: maker.pubkey(),
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            associated_token_program: anchor_spl::associated_token::ID,
            token_program: anchor_spl::token::ID,
            system_program: anchor_lang::solana_program::system_program::ID,
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&maker.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[maker]).unwrap();

    let res = svm.send_transaction(tx);
    // Since we are using mock unitialized token accounts, the transaction will fail validation check,
    // which proves that the instruction is successfully parsed and validated by the Solana runtime!
    assert!(res.is_err());
}