mod helpers;

use helpers::*;

#[test]
fn delist_returns_asset_to_maker_and_closes_listing() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 1_000_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, price),
    ));

    let ix = delist_ix(&maker.pubkey(), &asset.pubkey(), None, &setup.name);
    assert_ok(send_ix(&mut svm, &[&maker], ix));

    let asset_data = fetch_asset(&svm, &asset.pubkey());
    assert_eq!(asset_data.owner, maker.pubkey());

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    assert!(!account_exists(&svm, &listing_pk));
}

#[test]
fn delist_rejects_non_maker() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let attacker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 1_000_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, price),
    ));

    let ix = delist_ix(&attacker.pubkey(), &asset.pubkey(), None, &setup.name);
    let result = send_ix(&mut svm, &[&attacker], ix);
    assert!(result.is_err(), "non-maker must not be able to delist");
}
