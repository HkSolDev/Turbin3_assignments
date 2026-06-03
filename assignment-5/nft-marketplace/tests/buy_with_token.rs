mod helpers;

use helpers::*;

#[test]
fn buy_with_token_transfers_nft_pays_fees_and_mints_rewards() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let taker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let payment_mint = create_spl_mint(
        &mut svm,
        &setup.admin,
        &setup.admin.pubkey(),
        PAYMENT_MINT_DECIMALS,
    );
    let taker_payment_ata = fund_user_with_tokens(
        &mut svm,
        &setup.admin,
        &payment_mint.pubkey(),
        &setup.admin,
        &taker.pubkey(),
        INITIAL_PAYMENT_TOKENS,
    );

    let price = PAYMENT_TOKEN_UNIT.checked_mul(100).unwrap();
    assert_ok(send_ix(
        &mut svm,
        &[&maker],
        list_ix(
            &maker.pubkey(),
            &asset.pubkey(),
            None,
            Some(payment_mint.pubkey()),
            price,
        ),
    ));

    let ix = buy_with_token_ix(
        &taker.pubkey(),
        &maker.pubkey(),
        &setup.name,
        &asset.pubkey(),
        None,
        &payment_mint.pubkey(),
    );
    assert_ok(send_ix(&mut svm, &[&taker], ix));

    let fee = price
        .checked_mul(DEFAULT_FEE_BPS as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap();
    let maker_amount = price.checked_sub(fee).unwrap();

    let maker_payment_ata = ata(&maker.pubkey(), &payment_mint.pubkey());
    let treasury_payment_ata = ata(&setup.treasury, &payment_mint.pubkey());

    assert_eq!(token_balance(&svm, &maker_payment_ata), maker_amount);
    assert_eq!(token_balance(&svm, &treasury_payment_ata), fee);
    assert_eq!(
        token_balance(&svm, &taker_payment_ata),
        INITIAL_PAYMENT_TOKENS.checked_sub(price).unwrap()
    );

    assert_eq!(fetch_asset(&svm, &asset.pubkey()).owner, taker.pubkey());

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    assert!(!account_exists(&svm, &listing_pk));

    let taker_rewards_ata = ata(&taker.pubkey(), &setup.rewards_mint);
    assert_eq!(token_balance(&svm, &taker_rewards_ata), price);
}
