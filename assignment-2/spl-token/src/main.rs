use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::create_account, // Back to the standard way!

    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use spl_token::{
    ID as TOKEN_PROGRAM_ID,                          // Standard way to get the ID
    instruction::{initialize_mint, mint_to_checked}, // Standard Token program
    state::{Account, Mint},
};
#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Starting Program ---"); // ADD THIS
    let fee_payer = Keypair::new();
    let mint = Keypair::new();

    let client = RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::processed(),
    );

    println!("Requesting Airdrop for: {}...", fee_payer.pubkey());
    let airdrop_signature = client
        .request_airdrop(&fee_payer.pubkey(), 2_000_000_000) // 2 SOL in lamports
        .await?;

    println!("Airdrop requested. Waiting for confirmation...");
    loop {
        let confirmed = client.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }
    println!("Airdrop Confirmed!");

    println!("Fetching latest blockhash and rent...");
    let latest_blockhash = client
        .get_latest_blockhash()
        .await
        .expect("Did not able to get hte latest block-hash");

    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(82)
        .await
        .expect("error in get the rent");

    println!("Building and sending Mint transaction...");

    let tx = Transaction::new_signed_with_payer(
        &[
            create_account(
                &fee_payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &TOKEN_PROGRAM_ID,
            ),
            initialize_mint(
                &TOKEN_PROGRAM_ID,
                &mint.pubkey(),
                &fee_payer.pubkey(),
                Some(&fee_payer.pubkey()),
                9,
            )?,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    let transaction_signature = client.send_and_confirm_transaction(&tx).await?;
    println!("Mint Transaction Confirmed!");

    let mint_acc = client.get_account(&mint.pubkey()).await?;

    let mint_data = Mint::unpack_unchecked(&mint_acc.data)?;

    println!("Mint Address: {}", mint.pubkey());
    println!("Mint Account: {:#?}", mint_data);
    println!("\nTransaction Signature: {}", transaction_signature);

    let latest_blockhash = client
        .get_latest_blockhash()
        .await
        .expect("Did not able to get hte latest block-hash");

    println!("Building and sending ATA transaction...");
    let tx = Transaction::new_signed_with_payer(
        &[create_associated_token_account(
            &fee_payer.pubkey(),
            &fee_payer.pubkey(),
            &mint.pubkey(),
            &TOKEN_PROGRAM_ID,
        )],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        latest_blockhash,
    );

    let tx_signature = client.send_and_confirm_transaction(&tx).await?;
    println!("ATA Transaction Confirmed!");
    let ata_address = get_associated_token_address(&fee_payer.pubkey(), &mint.pubkey());
    let token_acc = client.get_account(&ata_address).await?;

    println!("Unpacking Token Account...");
    let token_data = Account::unpack(&token_acc.data)?;
    println!(
        "Token Account Balance BEFORE minting: {}",
        token_data.amount
    );

    println!("\nTransaction Signature: {}", tx_signature);

    let latest_blockHash = client.get_latest_blockhash().await?;

    println!("Building and sending MintTo transaction...");
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_checked(
            &TOKEN_PROGRAM_ID,
            &mint.pubkey(),
            &ata_address,
            &fee_payer.pubkey(),
            &[&fee_payer.pubkey()],
            1000000000000,
            9,
        )?],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        latest_blockHash,
    );

    let mint_tx_signature = client.send_and_confirm_transaction(&tx).await?;
    println!(
        "MintTo Transaction Confirmed! Signature: {}",
        mint_tx_signature
    );

    println!("Fetching updated Token Account...");
    let token_acc_final = client.get_account(&ata_address).await?;
    let token_data_final = Account::unpack(&token_acc_final.data)?;

    println!("--- FINAL RESULTS ---");
    println!("Mint Address: {}", mint.pubkey());
    println!("ATA Address:  {}", ata_address);
    println!("Final Balance: {} tokens", token_data_final.amount);
    println!("----------------------");

    Ok(())
}
