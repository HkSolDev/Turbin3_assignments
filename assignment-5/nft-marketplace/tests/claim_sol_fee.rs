mod helpers;

use helpers::*;
use nft_marketplace::error::MarketplaceError;

#[test]
fn admin_can_claim_sol_fees() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let taker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 1_000_000_000;
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

    let recipient = Keypair::new();
    let ix = claim_sol_fee_ix(&setup.admin.pubkey(), &setup.name, &recipient.pubkey(), fee);
    assert_ok(send_ix(&mut svm, &[&setup.admin], ix));

    assert_eq!(sol_balance(&svm, &setup.treasury), 0);
    assert_eq!(sol_balance(&svm, &recipient.pubkey()), fee);
}

#[test]
fn non_admin_cannot_claim_sol_fees() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let taker = funded_keypair(&mut svm);
    let attacker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 1_000_000_000;
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

    let ix = claim_sol_fee_ix(&attacker.pubkey(), &setup.name, &attacker.pubkey(), fee);
    let result = send_ix(&mut svm, &[&attacker], ix);
    assert!(result.is_err(), "non-admin claim must fail");
}

#[test]
fn claim_more_than_treasury_balance_fails() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let recipient = Keypair::new();

    let ix = claim_sol_fee_ix(
        &setup.admin.pubkey(),
        &setup.name,
        &recipient.pubkey(),
        1_000_000_000,
    );
    assert_marketplace_error(
        send_ix(&mut svm, &[&setup.admin], ix),
        MarketplaceError::InsufficientTreasuryFunds,
    );
}
