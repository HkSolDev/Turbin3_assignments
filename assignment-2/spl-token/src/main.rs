use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};

use mpl_token_metadata::{
    ID as METAPLEX_PROGRAM_ID, instructions::CreateMetadataAccountV3Builder, types::DataV2,
};
use spl_token::{
    ID as TOKEN_PROGRAM_ID,
    instruction::{initialize_mint, mint_to_checked},
    state::Mint,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Starting Token-2022 Metadata Program ---");
    let fee_payer = Keypair::new();
    let mint = Keypair::new();

    let client = RpcClient::new_with_commitment(
        String::from("http://localhost:8899"),
        CommitmentConfig::processed(),
    );

    // 1. Airdrop
    println!("Requesting Airdrop for: {}...", fee_payer.pubkey());
    let airdrop_signature = client
        .request_airdrop(&fee_payer.pubkey(), 2_000_000_000)
        .await?;
    client.confirm_transaction(&airdrop_signature).await?;
    println!("Airdrop Confirmed!");

    // 2. Define Metadata
    let name = "Solana Dev Token".to_string();
    let symbol = "SDT".to_string();
    let uri = "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSynmapbgNkV8W_3GOc9GasCnHI5m3rCdN-iw&s".to_string();

    // let metadata = TokenMetadata {
    //     name: name.clone(),
    //     symbol: symbol.clone(),
    //     uri: uri.clone(),
    //     update_authority: Some(fee_payer.pubkey()).try_into()?,
    //     mint: mint.pubkey(),
    //     ..Default::default()
    // };
    // 3. Calculate Space and Rent
    // Base Mint (82) + Padding (83) + Extension MetadataPointer (68) + Metadata size
    let mint_len = Mint::LEN;
    // let metadata_len = VariableLenPack::get_packed_len(&metadata)?;
    let total_len = mint_len;

    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(total_len)
        .await?;

    println!("Building and sending Mint + Metadata transaction...");
    let latest_blockhash = client.get_latest_blockhash().await?;

    let (metadata_pda, _bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            METAPLEX_PROGRAM_ID.as_ref(),
            mint.pubkey().as_ref(),
        ],
        &METAPLEX_PROGRAM_ID,
    );

    let create_metadata_ix = CreateMetadataAccountV3Builder::new()
        .metadata(metadata_pda)
        .mint(mint.pubkey())
        .mint_authority(fee_payer.pubkey())
        .payer(fee_payer.pubkey())
        .update_authority(fee_payer.pubkey(), true) // The 'true' means they can update it later
        .data(DataV2 {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        })
        .is_mutable(true)
        .instruction();

    // 4. Build Instructions
    let tx = Transaction::new_signed_with_payer(
        &[
            // Allocate space
            create_account(
                &fee_payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                total_len as u64,
                &TOKEN_PROGRAM_ID,
            ),
            // Initialize Mint
            initialize_mint(
                &TOKEN_PROGRAM_ID,
                &mint.pubkey(),
                &fee_payer.pubkey(),
                Some(&fee_payer.pubkey()),
                9,
            )?,
            create_metadata_ix,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint],
        latest_blockhash,
    );

    client.send_and_confirm_transaction(&tx).await?;
    println!("Mint and Metadata Initialized!");

    // // 5. Create ATA
    println!("Creating ATA...");
    let ata_address = get_associated_token_address_with_program_id(
        &fee_payer.pubkey(),
        &mint.pubkey(),
        &TOKEN_PROGRAM_ID,
    );

    let latest_blockhash = client.get_latest_blockhash().await?;
    let ata_tx = Transaction::new_signed_with_payer(
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
    client.send_and_confirm_transaction(&ata_tx).await?;
    println!("ATA Created: {}", ata_address);

    println!("Mint Address: {}", mint.pubkey());

    // 6. Mint Tokens
    println!("Minting tokens...");
    let latest_blockhash = client.get_latest_blockhash().await?;
    let mint_to_tx = Transaction::new_signed_with_payer(
        &[mint_to_checked(
            &TOKEN_PROGRAM_ID,
            &mint.pubkey(),
            &ata_address,
            &fee_payer.pubkey(),
            &[&fee_payer.pubkey()],
            1_000_000_000, // 1 token
            9,
        )?],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        latest_blockhash,
    );
    client.send_and_confirm_transaction(&mint_to_tx).await?;

    println!("--- FINAL RESULTS ---");
    println!("Mint Address: {}", mint.pubkey());
    println!("ATA Address:  {}", ata_address);
    println!("Token-2022 Program: {}", METAPLEX_PROGRAM_ID);
    println!("----------------------");

    Ok(())
}
