mod helpers;

use helpers::*;

#[test]
fn accept_sol_offer_settles_payment_and_transfers_nft() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let buyer = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let list_price: u64 = 1_000_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, list_price),
    ));

    let offer_amount: u64 = 800_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&buyer],
        make_sol_offer_ix(&buyer.pubkey(), &asset.pubkey(), offer_amount),
    ));

    let initial_treasury = sol_balance(&svm, &setup.treasury);
    let initial_maker = sol_balance(&svm, &maker.pubkey());

    let ix = accept_sol_offer_ix(
        &maker.pubkey(),
        &buyer.pubkey(),
        &setup.name,
        &asset.pubkey(),
        None,
    );
    assert_ok(send_ix(&mut svm, &[&maker], ix));

    let fee = (offer_amount as u128)
        .checked_mul(DEFAULT_FEE_BPS as u128)
        .unwrap()
        .checked_div(10_000)
        .unwrap() as u64;

    let maker_amount = offer_amount.checked_sub(fee).unwrap();

    let treasury_diff = sol_balance(&svm, &setup.treasury)
        .checked_sub(initial_treasury)
        .unwrap();
    assert_eq!(treasury_diff, fee);

    let expected_maker_post = initial_maker
        .checked_add(maker_amount)
        .unwrap()
        .checked_sub(10_000)
        .unwrap();
    assert!(sol_balance(&svm, &maker.pubkey()) >= expected_maker_post);

    assert_eq!(fetch_asset(&svm, &asset.pubkey()).owner, buyer.pubkey());

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    let (offer_pk, _) = offer_pda(&asset.pubkey(), &buyer.pubkey());
    assert!(!account_exists(&svm, &listing_pk));
    assert!(!account_exists(&svm, &offer_pk));
}
