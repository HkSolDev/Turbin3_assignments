#![allow(dead_code)]

pub use solana_sdk::signature::{Keypair, Signer};

use {
    anchor_lang::{
        prelude::*, system_program::ID as SYSTEM_PROGRAM_ID, InstructionData, ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{
            get_associated_token_address, spl_associated_token_account,
            ID as ASSOCIATED_TOKEN_PROGRAM_ID,
        },
        token::{spl_token, ID as TOKEN_PROGRAM_ID},
    },
    litesvm::{types::TransactionResult, LiteSVM},
    mpl_core::{
        accounts::{BaseAssetV1, BaseCollectionV1},
        instructions::{CreateCollectionV1Builder, CreateV1Builder},
        ID as MPL_CORE_PROGRAM_ID,
    },
    nft_marketplace::{
        accounts as ix_accounts,
        error::MarketplaceError,
        instruction as ix_data,
        state::{Listing, MarketPlace, Offer},
        LISTING, MARKETPLACE, OFFER, OFFER_VAULT, REWARDS_MINT, TREASURY,
    },
    solana_sdk::{
        instruction::{Instruction, InstructionError},
        message::Message,
        program_pack::Pack,
        rent::Rent,
        system_instruction,
        transaction::{Transaction, TransactionError},
    },
};

pub const INITIAL_USER_LAMPORTS: u64 = 5_000_000_000;
pub const DEFAULT_FEE_BPS: u16 = 500;
pub const MARKETPLACE_NAME: &str = "Test Marketplace";
pub const COLLECTION_NAME: &str = "Test Collection";
pub const COLLECTION_URI: &str = "https://example.com/collection.json";
pub const ASSET_NAME: &str = "Test Asset";
pub const ASSET_URI: &str = "https://example.com/asset.json";
pub const PAYMENT_MINT_DECIMALS: u8 = 6;
pub const PAYMENT_TOKEN_UNIT: u64 = 10u64.pow(PAYMENT_MINT_DECIMALS as u32);
pub const INITIAL_PAYMENT_TOKENS: u64 = 1_000u64
    .checked_mul(PAYMENT_TOKEN_UNIT)
    .expect("INITIAL_PAYMENT_TOKENS overflow");

pub fn init_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();

    svm.add_program_from_file(
        nft_marketplace::ID,
        "../../target/deploy/nft_marketplace.so",
    )
    .unwrap();

    svm.add_program_from_file(MPL_CORE_PROGRAM_ID, "../../target/deploy/mpl_core.so")
        .unwrap_or_else(|e: std::io::Error| {
            panic!(
                "load mpl-core failed: {e:?}\nrun `bash scripts/setup.sh` once to dump mpl-core from mainnet"
            )
        });

    svm
}

pub fn airdrop(svm: &mut LiteSVM, recipient: &Pubkey, lamports: u64) {
    svm.airdrop(recipient, lamports).unwrap();
}

pub fn funded_keypair(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    airdrop(svm, &kp.pubkey(), INITIAL_USER_LAMPORTS);
    kp
}

pub fn marketplace_pda(name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MARKETPLACE, name.as_bytes()], &nft_marketplace::ID)
}

pub fn treasury_pda(marketplace: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY, marketplace.as_ref()], &nft_marketplace::ID)
}

pub fn rewards_mint_pda(marketplace: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARDS_MINT, marketplace.as_ref()], &nft_marketplace::ID)
}

pub fn listing_pda(asset: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[LISTING, asset.as_ref()], &nft_marketplace::ID)
}

pub fn offer_pda(asset: &Pubkey, buyer: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[OFFER, asset.as_ref(), buyer.as_ref()],
        &nft_marketplace::ID,
    )
}

pub fn offer_vault_pda(asset: &Pubkey, buyer: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[OFFER_VAULT, asset.as_ref(), buyer.as_ref()],
        &nft_marketplace::ID,
    )
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(owner, mint)
}

pub fn init_marketplace_ix(admin: &Pubkey, name: &str, fee: u16) -> Instruction {
    let (marketplace, _) = marketplace_pda(name);
    let (treasury, _) = treasury_pda(&marketplace);
    let (rewards_mint, _) = rewards_mint_pda(&marketplace);

    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::Init {
            admin: *admin,
            marketplace,
            treasury,
            rewards_mint,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::Init {
            name: name.to_string(),
            fee,
        }
        .data(),
    }
}

pub fn list_ix(
    maker: &Pubkey,
    asset: &Pubkey,
    collection: Option<Pubkey>,
    payment_mint: Option<Pubkey>,
    price: u64,
) -> Instruction {
    let (listing, _) = listing_pda(asset);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::List {
            maker: *maker,
            asset: *asset,
            collection,
            listing,
            payment_mint,
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::List { price }.data(),
    }
}

pub fn delist_ix(
    maker: &Pubkey,
    asset: &Pubkey,
    collection: Option<Pubkey>,
    marketplace_name: &str,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (listing, _) = listing_pda(asset);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::Delist {
            maker: *maker,
            marketplace,
            asset: *asset,
            collection,
            listing,
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::Delist {}.data(),
    }
}

pub fn buy_with_sol_ix(
    taker: &Pubkey,
    maker: &Pubkey,
    marketplace_name: &str,
    asset: &Pubkey,
    collection: Option<Pubkey>,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (treasury, _) = treasury_pda(&marketplace);
    let (rewards_mint, _) = rewards_mint_pda(&marketplace);
    let (listing, _) = listing_pda(asset);
    let taker_reward_ata = ata(taker, &rewards_mint);

    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::BuyWithSol {
            taker: *taker,
            maker: *maker,
            marketplace,
            asset: *asset,
            collection,
            listing,
            treasury,
            rewards_mint,
            taker_reward_ata,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            mpl_core_program: MPL_CORE_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::BuyWithSol {}.data(),
    }
}

pub fn buy_with_token_ix(
    taker: &Pubkey,
    maker: &Pubkey,
    marketplace_name: &str,
    asset: &Pubkey,
    collection: Option<Pubkey>,
    payment_mint: &Pubkey,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (treasury, _) = treasury_pda(&marketplace);
    let (rewards_mint, _) = rewards_mint_pda(&marketplace);
    let (listing, _) = listing_pda(asset);
    let taker_payment_ata = ata(taker, payment_mint);
    let maker_payment_ata = ata(maker, payment_mint);
    let treasury_payment_ata = ata(&treasury, payment_mint);
    let taker_reward_ata = ata(taker, &rewards_mint);

    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::BuyWithToken {
            taker: *taker,
            maker: *maker,
            marketplace,
            treasury,
            asset: *asset,
            collection,
            listing,
            payment_mint: *payment_mint,
            rewards_mint,
            taker_payment_ata,
            maker_payment_ata,
            treasury_payment_ata,
            taker_reward_ata,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            mpl_core_program: MPL_CORE_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::BuyWithToken {}.data(),
    }
}

pub fn claim_sol_fee_ix(
    admin: &Pubkey,
    marketplace_name: &str,
    recipient: &Pubkey,
    amount: u64,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (treasury, _) = treasury_pda(&marketplace);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::ClaimSolFee {
            admin: *admin,
            marketplace,
            treasury,
            recipient: *recipient,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::ClaimSolFee { amount }.data(),
    }
}

pub fn claim_token_fee_ix(
    admin: &Pubkey,
    marketplace_name: &str,
    recipient: &Pubkey,
    payment_mint: &Pubkey,
    amount: u64,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (treasury, _) = treasury_pda(&marketplace);
    let treasury_payment_ata = ata(&treasury, payment_mint);
    let recipient_payment_ata = ata(recipient, payment_mint);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::ClaimTokenFee {
            admin: *admin,
            marketplace,
            treasury,
            treasury_payment_ata,
            payment_mint: *payment_mint,
            recipient: *recipient,
            recipient_payment_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::ClaimTokenFee { amount }.data(),
    }
}

pub fn make_sol_offer_ix(buyer: &Pubkey, asset: &Pubkey, amount: u64) -> Instruction {
    let (offer, _) = offer_pda(asset, buyer);
    let (offer_vault, _) = offer_vault_pda(asset, buyer);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::MakeSolOffer {
            buyer: *buyer,
            asset: *asset,
            offer,
            offer_vault,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::MakeSolOffer { amount }.data(),
    }
}

pub fn accept_sol_offer_ix(
    maker: &Pubkey,
    buyer: &Pubkey,
    marketplace_name: &str,
    asset: &Pubkey,
    collection: Option<Pubkey>,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (treasury, _) = treasury_pda(&marketplace);
    let (listing, _) = listing_pda(asset);
    let (offer, _) = offer_pda(asset, buyer);
    let (offer_vault, _) = offer_vault_pda(asset, buyer);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::AcceptSolOffer {
            maker: *maker,
            buyer: *buyer,
            marketplace,
            asset: *asset,
            collection,
            listing,
            offer,
            offer_vault,
            treasury,
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::AcceptSolOffer {}.data(),
    }
}

pub fn cancel_sol_offer_ix(buyer: &Pubkey, asset: &Pubkey) -> Instruction {
    let (offer, _) = offer_pda(asset, buyer);
    let (offer_vault, _) = offer_vault_pda(asset, buyer);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::CancelSolOffer {
            buyer: *buyer,
            asset: *asset,
            offer,
            offer_vault,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::CancelSolOffer {}.data(),
    }
}

pub fn make_token_offer_ix(
    buyer: &Pubkey,
    asset: &Pubkey,
    payment_mint: &Pubkey,
    amount: u64,
) -> Instruction {
    let (offer, _) = offer_pda(asset, buyer);
    let buyer_payment_ata = ata(buyer, payment_mint);
    let offer_vault_ata = ata(&offer, payment_mint);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::MakeTokenOffer {
            buyer: *buyer,
            asset: *asset,
            offer,
            payment_mint: *payment_mint,
            buyer_payment_ata,
            offer_vault_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::MakeTokenOffer { amount }.data(),
    }
}

pub fn accept_token_offer_ix(
    maker: &Pubkey,
    buyer: &Pubkey,
    marketplace_name: &str,
    asset: &Pubkey,
    collection: Option<Pubkey>,
    payment_mint: &Pubkey,
) -> Instruction {
    let (marketplace, _) = marketplace_pda(marketplace_name);
    let (treasury, _) = treasury_pda(&marketplace);
    let (rewards_mint, _) = rewards_mint_pda(&marketplace);
    let (listing, _) = listing_pda(asset);
    let (offer, _) = offer_pda(asset, buyer);
    let offer_vault_ata = ata(&offer, payment_mint);
    let maker_payment_ata = ata(maker, payment_mint);
    let treasury_payment_ata = ata(&treasury, payment_mint);
    let buyer_reward_ata = ata(buyer, &rewards_mint);

    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::AcceptTokenOffer {
            maker: *maker,
            buyer: *buyer,
            marketplace,
            treasury,
            asset: *asset,
            collection,
            listing,
            offer,
            payment_mint: *payment_mint,
            offer_vault_ata,
            maker_payment_ata,
            treasury_payment_ata,
            rewards_mint,
            buyer_reward_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
            mpl_core_program: MPL_CORE_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::AcceptTokenOffer {}.data(),
    }
}

pub fn cancel_token_offer_ix(buyer: &Pubkey, asset: &Pubkey, payment_mint: &Pubkey) -> Instruction {
    let (offer, _) = offer_pda(asset, buyer);
    let buyer_payment_ata = ata(buyer, payment_mint);
    let offer_vault_ata = ata(&offer, payment_mint);
    Instruction {
        program_id: nft_marketplace::ID,
        accounts: ix_accounts::CancelTokenOffer {
            buyer: *buyer,
            asset: *asset,
            offer,
            payment_mint: *payment_mint,
            buyer_payment_ata,
            offer_vault_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix_data::CancelTokenOffer {}.data(),
    }
}

pub fn send_ix(svm: &mut LiteSVM, signers: &[&Keypair], ix: Instruction) -> TransactionResult {
    send_ixs(svm, signers, &[ix])
}

pub fn send_ixs(svm: &mut LiteSVM, signers: &[&Keypair], ixs: &[Instruction]) -> TransactionResult {
    let payer = signers.first().expect("at least one signer required");
    let message = Message::new(ixs, Some(&payer.pubkey()));
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new(signers, message, blockhash);
    let result = svm.send_transaction(tx);
    svm.expire_blockhash();
    result
}

pub fn fetch_marketplace(svm: &LiteSVM, marketplace: &Pubkey) -> MarketPlace {
    let account = svm
        .get_account(marketplace)
        .expect("marketplace account missing");
    MarketPlace::try_deserialize(&mut account.data.as_ref()).expect("decode MarketPlace")
}

pub fn fetch_listing(svm: &LiteSVM, listing: &Pubkey) -> Listing {
    let account = svm.get_account(listing).expect("listing account missing");
    Listing::try_deserialize(&mut account.data.as_ref()).expect("decode Listing")
}

pub fn fetch_offer(svm: &LiteSVM, offer: &Pubkey) -> Offer {
    let account = svm.get_account(offer).expect("offer account missing");
    Offer::try_deserialize(&mut account.data.as_ref()).expect("decode Offer")
}

pub fn fetch_asset(svm: &LiteSVM, asset: &Pubkey) -> BaseAssetV1 {
    let account = svm.get_account(asset).expect("asset account missing");
    BaseAssetV1::from_bytes(&account.data).expect("decode BaseAssetV1")
}

pub fn fetch_collection(svm: &LiteSVM, collection: &Pubkey) -> BaseCollectionV1 {
    let account = svm
        .get_account(collection)
        .expect("collection account missing");
    BaseCollectionV1::from_bytes(&account.data).expect("decode BaseCollectionV1")
}

pub fn token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    svm.get_account(ata)
        .filter(|acc| acc.data.len() >= 72)
        .map(|acc| {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&acc.data[64..72]);
            u64::from_le_bytes(buf)
        })
        .unwrap_or(0)
}

pub fn sol_balance(svm: &LiteSVM, pubkey: &Pubkey) -> u64 {
    svm.get_account(pubkey).map(|a| a.lamports).unwrap_or(0)
}

pub fn account_exists(svm: &LiteSVM, pubkey: &Pubkey) -> bool {
    svm.get_account(pubkey)
        .map(|a| !a.data.is_empty() || a.lamports > 0)
        .unwrap_or(false)
}

pub fn assert_ok(result: TransactionResult) {
    if let Err(failed) = result {
        panic!("expected success, got error:\n{:#?}", failed.meta.logs);
    }
}

pub fn assert_marketplace_error(result: TransactionResult, expected: MarketplaceError) {
    let expected_code: u32 = expected.into();
    match result {
        Ok(meta) => panic!(
            "expected `{expected:?}` (code {expected_code}), but transaction succeeded.\n\
             logs:\n{:#?}",
            meta.logs
        ),
        Err(failed) => match failed.err {
            TransactionError::InstructionError(_, InstructionError::Custom(code)) => assert_eq!(
                code, expected_code,
                "expected `{expected:?}` (code {expected_code}), got code {code}.\n\
                 logs:\n{:#?}",
                failed.meta.logs
            ),
            other => panic!(
                "expected `{expected:?}`, got structural error: {other:?}.\n\
                 logs:\n{:#?}",
                failed.meta.logs
            ),
        },
    }
}

pub struct MarketplaceSetup {
    pub admin: Keypair,
    pub name: String,
    pub fee: u16,
    pub marketplace: Pubkey,
    pub treasury: Pubkey,
    pub rewards_mint: Pubkey,
}

pub fn setup_marketplace(svm: &mut LiteSVM, fee: u16) -> MarketplaceSetup {
    let admin = funded_keypair(svm);
    let name = MARKETPLACE_NAME.to_string();
    let (marketplace, _) = marketplace_pda(&name);
    let (treasury, _) = treasury_pda(&marketplace);
    let (rewards_mint, _) = rewards_mint_pda(&marketplace);

    let ix = init_marketplace_ix(&admin.pubkey(), &name, fee);
    assert_ok(send_ix(svm, &[&admin], ix));

    MarketplaceSetup {
        admin,
        name,
        fee,
        marketplace,
        treasury,
        rewards_mint,
    }
}

pub fn create_mpl_collection(svm: &mut LiteSVM, payer: &Keypair) -> Keypair {
    let collection = Keypair::new();
    let ix = CreateCollectionV1Builder::new()
        .collection(collection.pubkey())
        .payer(payer.pubkey())
        .update_authority(Some(payer.pubkey()))
        .name(COLLECTION_NAME.to_string())
        .uri(COLLECTION_URI.to_string())
        .instruction();
    assert_ok(send_ix(svm, &[payer, &collection], ix));
    collection
}

pub fn create_mpl_asset(svm: &mut LiteSVM, owner: &Keypair) -> Keypair {
    let asset = Keypair::new();
    let ix = CreateV1Builder::new()
        .asset(asset.pubkey())
        .payer(owner.pubkey())
        .name(ASSET_NAME.to_string())
        .uri(ASSET_URI.to_string())
        .instruction();
    assert_ok(send_ix(svm, &[owner, &asset], ix));
    asset
}

pub fn create_spl_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint_authority: &Pubkey,
    decimals: u8,
) -> Keypair {
    let mint = Keypair::new();
    let rent: Rent = svm.get_sysvar();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &TOKEN_PROGRAM_ID,
    );

    let init_mint_ix = spl_token::instruction::initialize_mint2(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        mint_authority,
        None,
        decimals,
    )
    .unwrap();

    assert_ok(send_ixs(
        svm,
        &[payer, &mint],
        &[create_account_ix, init_mint_ix],
    ));
    mint
}

pub fn create_ata(svm: &mut LiteSVM, payer: &Keypair, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let ata_pk = ata(owner, mint);
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        owner,
        mint,
        &TOKEN_PROGRAM_ID,
    );
    assert_ok(send_ix(svm, &[payer], ix));
    ata_pk
}

pub fn mint_to_ata(
    svm: &mut LiteSVM,
    mint: &Pubkey,
    mint_authority: &Keypair,
    dest: &Pubkey,
    amount: u64,
) {
    let ix = spl_token::instruction::mint_to(
        &TOKEN_PROGRAM_ID,
        mint,
        dest,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    assert_ok(send_ix(svm, &[mint_authority], ix));
}

pub fn fund_user_with_tokens(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    mint_authority: &Keypair,
    user: &Pubkey,
    amount: u64,
) -> Pubkey {
    let user_ata = create_ata(svm, payer, user, mint);
    mint_to_ata(svm, mint, mint_authority, &user_ata, amount);
    user_ata
}
