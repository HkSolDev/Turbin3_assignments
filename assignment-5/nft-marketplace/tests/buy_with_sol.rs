mod helpers;

use helpers::*;

#[test]
fn buy_with_sol_transfers_nft_pays_fees_and_mints_rewards() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let taker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 1_000_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, price),
    ));

    let initial_treasury = sol_balance(&svm, &setup.treasury);

    let buy_ix = buy_with_sol_ix(
        &taker.pubkey(),
        &maker.pubkey(),
        &setup.name,
        &asset.pubkey(),
        None,
    );
    assert_ok(send_ix(&mut svm, &[&taker], buy_ix));

    let fee = price
        .checked_mul(DEFAULT_FEE_BPS as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap();

    let treasury_diff = sol_balance(&svm, &setup.treasury)
        .checked_sub(initial_treasury)
        .unwrap();
    assert_eq!(treasury_diff, fee);

    let asset_data = fetch_asset(&svm, &asset.pubkey());
    assert_eq!(asset_data.owner, taker.pubkey());

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    assert!(!account_exists(&svm, &listing_pk));

    let taker_rewards_ata = ata(&taker.pubkey(), &setup.rewards_mint);
    assert_eq!(token_balance(&svm, &taker_rewards_ata), price);
}

#[test]
fn buy_with_sol_at_zero_fee_marketplace_sends_full_price_to_maker() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, 0);
    let maker = funded_keypair(&mut svm);
    let taker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 500_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(&maker.pubkey(), &asset.pubkey(), None, None, price),
    ));

    let initial_treasury = sol_balance(&svm, &setup.treasury);

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

    assert_eq!(sol_balance(&svm, &setup.treasury), initial_treasury);
    assert_eq!(fetch_asset(&svm, &asset.pubkey()).owner, taker.pubkey());
}
