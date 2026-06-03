mod helpers;

use helpers::*;

#[test]
fn accept_token_offer_settles_payment_transfers_nft_and_mints_rewards() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let buyer = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let payment_mint = create_spl_mint(
        &mut svm,
        &setup.admin,
        &setup.admin.pubkey(),
        PAYMENT_MINT_DECIMALS,
    );
    fund_user_with_tokens(
        &mut svm,
        &setup.admin,
        &payment_mint.pubkey(),
        &setup.admin,
        &buyer.pubkey(),
        INITIAL_PAYMENT_TOKENS,
    );

    let list_price = PAYMENT_TOKEN_UNIT.checked_mul(100).unwrap();
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(
            &maker.pubkey(),
            &asset.pubkey(),
            None,
            Some(payment_mint.pubkey()),
            list_price,
        ),
    ));

    let offer_amount = PAYMENT_TOKEN_UNIT.checked_mul(80).unwrap();
    assert_ok(send_ix(
        &mut svm,
        &[&buyer],
        make_token_offer_ix(
            &buyer.pubkey(),
            &asset.pubkey(),
            &payment_mint.pubkey(),
            offer_amount,
        ),
    ));

    let ix = accept_token_offer_ix(
        &maker.pubkey(),
        &buyer.pubkey(),
        &setup.name,
        &asset.pubkey(),
        None,
        &payment_mint.pubkey(),
    );
    assert_ok(send_ix(&mut svm, &[&maker], ix));

    let fee = offer_amount
        .checked_mul(DEFAULT_FEE_BPS as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap();
    let maker_amount = offer_amount.checked_sub(fee).unwrap();

    let maker_payment_ata = ata(&maker.pubkey(), &payment_mint.pubkey());
    let treasury_payment_ata = ata(&setup.treasury, &payment_mint.pubkey());
    assert_eq!(token_balance(&svm, &maker_payment_ata), maker_amount);
    assert_eq!(token_balance(&svm, &treasury_payment_ata), fee);

    assert_eq!(fetch_asset(&svm, &asset.pubkey()).owner, buyer.pubkey());

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    let (offer_pk, _) = offer_pda(&asset.pubkey(), &buyer.pubkey());
    assert!(!account_exists(&svm, &listing_pk));
    assert!(!account_exists(&svm, &offer_pk));

    let buyer_rewards_ata = ata(&buyer.pubkey(), &setup.rewards_mint);
    assert_eq!(token_balance(&svm, &buyer_rewards_ata), offer_amount);
}
