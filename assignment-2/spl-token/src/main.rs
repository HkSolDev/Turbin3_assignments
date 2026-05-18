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

use solana_sdk::signature::read_keypair_file;

use mpl_token_metadata::{
    ID as METAPLEX_PROGRAM_ID,
    instructions::{CreateMasterEditionV3Builder, CreateMetadataAccountV3Builder},
    types::DataV2,
};
use spl_token::{
    ID as TOKEN_PROGRAM_ID,
    instruction::{initialize_mint, mint_to_checked},
    state::Mint,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("--- Starting  Metadata Program ---");
    let fee_payer = read_keypair_file("/Users/mrblackghost/.config/solana/id.json")
        .expect("Couldn't find your wallet file");

    let mint = read_keypair_file("mint-keypair.json").unwrap_or_else(|_| {
        let new_key = Keypair::new();
        // You'll want to save this to a file later so you don't lose it!
        new_key
    });
    let client = RpcClient::new_with_commitment(
        String::from("https://api.devnet.solana.com"),
        CommitmentConfig::confirmed(),
    );

    let name = "Leo".to_string();
    let symbol = "LEO".to_string();
    let uri = "https://raw.githubusercontent.com/HkSolDev/Turbin3_assignments/refs/heads/main/assignment-2/spl-token/token-metadata.json".to_string();
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
            create_account(
                &fee_payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                total_len as u64,
                &TOKEN_PROGRAM_ID,
            ),
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

    // CREATE THE NFT ---
    println!("\n--- Starting NFT Creation ---");
    let nft_mint = read_keypair_file("nft-mint-keypair.json").unwrap_or_else(|_| {
        let new_key = Keypair::new();
        println!("Created new NFT keypair file: nft-mint-keypair.json");
        new_key
    }); // A new mint for the NFT

    // Calculate PDAs for NFT
    let (nft_metadata_pda, _bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            METAPLEX_PROGRAM_ID.as_ref(),
            nft_mint.pubkey().as_ref(),
        ],
        &METAPLEX_PROGRAM_ID,
    );
    let (nft_master_edition_pda, _bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            METAPLEX_PROGRAM_ID.as_ref(),
            nft_mint.pubkey().as_ref(),
            b"edition",
        ],
        &METAPLEX_PROGRAM_ID,
    );
    // 1. Calculate the NFT's ATA first
    let nft_ata = get_associated_token_address_with_program_id(
        &fee_payer.pubkey(),
        &nft_mint.pubkey(),
        &TOKEN_PROGRAM_ID,
    );

    // 2. Build Instructions
    let create_nft_metadata_ix = CreateMetadataAccountV3Builder::new()
        .metadata(nft_metadata_pda)
        .mint(nft_mint.pubkey())
        .mint_authority(fee_payer.pubkey())
        .payer(fee_payer.pubkey())
        .update_authority(fee_payer.pubkey(), true)
        .data(DataV2 {
            name: "Leo NFT".to_string(),
            symbol: "LEONFT".to_string(),
            uri: uri.clone(),             // You can use the same JSON or a new one
            seller_fee_basis_points: 500, // 5% royalty
            creators: None,
            collection: None,
            uses: None,
        })
        .is_mutable(true)
        .instruction();

    let create_master_edition_ix = CreateMasterEditionV3Builder::new()
        .edition(nft_master_edition_pda)
        .mint(nft_mint.pubkey())
        .update_authority(fee_payer.pubkey())
        .mint_authority(fee_payer.pubkey())
        .payer(fee_payer.pubkey())
        .metadata(nft_metadata_pda)
        .max_supply(0)
        .instruction();

    // 3. Send Transaction (Mint + Metadata + Master Edition)
    let latest_blockhash = client.get_latest_blockhash().await?;
    let nft_tx = Transaction::new_signed_with_payer(
        &[
            create_account(
                &fee_payer.pubkey(),
                &nft_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &TOKEN_PROGRAM_ID,
            ),
            initialize_mint(
                &TOKEN_PROGRAM_ID,
                &nft_mint.pubkey(),
                &fee_payer.pubkey(),
                Some(&fee_payer.pubkey()),
                0,
            )?, // 0 Decimals!
            create_nft_metadata_ix,
            create_associated_token_account(
                &fee_payer.pubkey(),
                &fee_payer.pubkey(),
                &nft_mint.pubkey(),
                &TOKEN_PROGRAM_ID,
            ),
            mint_to_checked(
                &TOKEN_PROGRAM_ID,
                &nft_mint.pubkey(),
                &nft_ata,
                &fee_payer.pubkey(),
                &[&fee_payer.pubkey()],
                1, // Mint exactly 1
                0, // 0 decimals
            )?,
            create_master_edition_ix,
        ],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &nft_mint],
        latest_blockhash,
    );
    client.send_and_confirm_transaction(&nft_tx).await?;

    println!("--- NFT FINAL RESULTS ---");
    println!("NFT Address: {}", nft_mint.pubkey());
    println!("----------------------");
    Ok(())
}
