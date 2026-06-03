mod helpers;

use helpers::*;

#[test]
fn cancel_token_offer_refunds_tokens_and_closes_offer() {
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
    assert_ok(send_ix(
        &mut svm,
        &[&buyer],
        make_token_offer_ix(
            &buyer.pubkey(),
            &asset_kp.pubkey(),
            &payment_mint.pubkey(),
            amount,
        ),
    ));

    let ix = cancel_token_offer_ix(&buyer.pubkey(), &asset_kp.pubkey(), &payment_mint.pubkey());
    assert_ok(send_ix(&mut svm, &[&buyer], ix));

    assert_eq!(token_balance(&svm, &buyer_ata), INITIAL_PAYMENT_TOKENS);

    let (offer_pk, _) = offer_pda(&asset_kp.pubkey(), &buyer.pubkey());
    assert!(!account_exists(&svm, &offer_pk));
}
