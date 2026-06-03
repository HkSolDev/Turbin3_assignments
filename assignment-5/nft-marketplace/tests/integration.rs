mod helpers;

use helpers::*;

#[test]
fn full_sale_lifecycle_list_buy_claim() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let taker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 2_000_000_000;
    let fee = price
        .checked_mul(DEFAULT_FEE_BPS as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap();

    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, price),
    ));

    assert_ok(send_ix(
        &mut svm,
        &[&taker],
        buy_with_sol_ix(
            &taker.pubkey(),
            &maker.pubkey(),
            &setup.name,
            &asset.pubkey(),
            None,
        ),
    ));

    assert_eq!(sol_balance(&svm, &setup.treasury), fee);
    assert_eq!(fetch_asset(&svm, &asset.pubkey()).owner, taker.pubkey());

    let recipient = Keypair::new();
    assert_ok(send_ix(
        &mut svm,
        &[&setup.admin],
        claim_sol_fee_ix(&setup.admin.pubkey(), &setup.name, &recipient.pubkey(), fee),
    ));

    assert_eq!(sol_balance(&svm, &setup.treasury), 0);
    assert_eq!(sol_balance(&svm, &recipient.pubkey()), fee);
}

#[test]
fn full_offer_lifecycle_list_offer_accept() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let buyer = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let list_price: u64 = 2_000_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, list_price),
    ));

    let offer_amount: u64 = 1_500_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&buyer],
        make_sol_offer_ix(&buyer.pubkey(), &asset.pubkey(), offer_amount),
    ));

    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        accept_sol_offer_ix(
            &maker.pubkey(),
            &buyer.pubkey(),
            &setup.name,
            &asset.pubkey(),
            None,
        ),
    ));

    assert_eq!(fetch_asset(&svm, &asset.pubkey()).owner, buyer.pubkey());

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    let (offer_pk, _) = offer_pda(&asset.pubkey(), &buyer.pubkey());
    assert!(!account_exists(&svm, &listing_pk));
    assert!(!account_exists(&svm, &offer_pk));
}
