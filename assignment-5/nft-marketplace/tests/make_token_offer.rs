mod helpers;

use helpers::*;

#[test]
fn make_token_offer_stores_state_and_locks_tokens() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let buyer = funded_keypair(&mut svm);
    let asset_kp = Keypair::new();

    let payment_mint = create_spl_mint(
        &mut svm,
        &setup.admin,
        &setup.admin.pubkey(),
        PAYMENT_MINT_DECIMALS,
    );
    let buyer_ata = fund_user_with_tokens(
        &mut svm,
        &setup.admin,
        &payment_mint.pubkey(),
        &setup.admin,
        &buyer.pubkey(),
        INITIAL_PAYMENT_TOKENS,
    );

    let amount = PAYMENT_TOKEN_UNIT.checked_mul(50).unwrap();
    let ix = make_token_offer_ix(
        &buyer.pubkey(),
        &asset_kp.pubkey(),
        &payment_mint.pubkey(),
        amount,
    );
    assert_ok(send_ix(&mut svm, &[&buyer], ix));

    let (offer_pk, _) = offer_pda(&asset_kp.pubkey(), &buyer.pubkey());
    let offer = fetch_offer(&svm, &offer_pk);
    assert_eq!(offer.buyer, buyer.pubkey());
    assert_eq!(offer.asset, asset_kp.pubkey());
    assert_eq!(offer.amount, amount);
    assert_eq!(offer.payment_mint, Some(payment_mint.pubkey()));

    let vault = ata(&offer_pk, &payment_mint.pubkey());
    assert_eq!(token_balance(&svm, &vault), amount);
    assert_eq!(
        token_balance(&svm, &buyer_ata),
        INITIAL_PAYMENT_TOKENS.checked_sub(amount).unwrap()
    );
}
