mod helpers;

use helpers::*;
use nft_marketplace::error::MarketplaceError;

#[test]
fn init_creates_marketplace_with_correct_fields() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);

    let mp = fetch_marketplace(&svm, &setup.marketplace);
    assert_eq!(mp.admin, setup.admin.pubkey());
    assert_eq!(mp.fee, DEFAULT_FEE_BPS);
    assert_eq!(mp.name, MARKETPLACE_NAME);
    assert_ne!(mp.bump, 0);
    assert_ne!(mp.treasury_bump, 0);
    assert_ne!(mp.rewards_bump, 0);
}

#[test]
fn init_rejects_empty_name() {
    let mut svm = init_svm();
    let admin = funded_keypair(&mut svm);
    let ix = init_marketplace_ix(&admin.pubkey(), "", DEFAULT_FEE_BPS);
    assert_marketplace_error(
        send_ix(&mut svm, &[&admin], ix),
        MarketplaceError::EmptyName,
    );
}

#[test]
fn init_rejects_name_too_long() {
    let mut svm = init_svm();
    let admin = funded_keypair(&mut svm);
    let long_name: String = "a".repeat(31);
    let ix = init_marketplace_ix(&admin.pubkey(), &long_name, DEFAULT_FEE_BPS);
    assert_marketplace_error(
        send_ix(&mut svm, &[&admin], ix),
        MarketplaceError::NameTooLong,
    );
}

#[test]
fn init_rejects_fee_too_high() {
    let mut svm = init_svm();
    let admin = funded_keypair(&mut svm);
    let ix = init_marketplace_ix(&admin.pubkey(), MARKETPLACE_NAME, 10_000);
    assert_marketplace_error(
        send_ix(&mut svm, &[&admin], ix),
        MarketplaceError::InvalidFee,
    );
}

#[test]
fn init_fails_when_called_twice_on_same_name() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let ix = init_marketplace_ix(&setup.admin.pubkey(), &setup.name, DEFAULT_FEE_BPS);
    let result = send_ix(&mut svm, &[&setup.admin], ix);
    assert!(
        result.is_err(),
        "second init must fail; marketplace PDA already in use"
    );
}
