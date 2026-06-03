mod helpers;

use helpers::*;

#[test]
fn cancel_sol_offer_refunds_buyer_and_closes_offer() {
    let mut svm = init_svm();
    let _setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let buyer = funded_keypair(&mut svm);
    let asset_kp = Keypair::new();

    let amount: u64 = 500_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&buyer],
        make_sol_offer_ix(&buyer.pubkey(), &asset_kp.pubkey(), amount),
    ));

    let pre_cancel_buyer = sol_balance(&svm, &buyer.pubkey());

    let ix = cancel_sol_offer_ix(&buyer.pubkey(), &asset_kp.pubkey());
    assert_ok(send_ix(&mut svm, &[&buyer], ix));

    let (offer_pk, _) = offer_pda(&asset_kp.pubkey(), &buyer.pubkey());
    assert!(!account_exists(&svm, &offer_pk));

    let post_cancel_buyer = sol_balance(&svm, &buyer.pubkey());
    let expected = pre_cancel_buyer
        .checked_add(amount)
        .unwrap()
        .checked_sub(10_000)
        .unwrap();
    assert!(
        post_cancel_buyer >= expected,
        "buyer should be refunded, pre={pre_cancel_buyer} post={post_cancel_buyer}"
    );
}

#[test]
fn non_buyer_cannot_cancel_offer() {
    let mut svm = init_svm();
    let _setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let buyer = funded_keypair(&mut svm);
    let attacker = funded_keypair(&mut svm);
    let asset_kp = Keypair::new();

    let amount: u64 = 500_000_000;
    assert_ok(send_ix(
        &mut svm,
        &[&buyer],
        make_sol_offer_ix(&buyer.pubkey(), &asset_kp.pubkey(), amount),
    ));

    let ix = cancel_sol_offer_ix(&attacker.pubkey(), &asset_kp.pubkey());
    let result = send_ix(&mut svm, &[&attacker], ix);
    assert!(result.is_err(), "non-buyer cancel must fail");
}
