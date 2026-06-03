mod helpers;

use helpers::*;
use nft_marketplace::error::MarketplaceError;

#[test]
fn make_sol_offer_stores_state_and_locks_funds() {
    let mut svm = init_svm();
    let _setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let buyer = funded_keypair(&mut svm);
    let asset_kp = Keypair::new();

    let amount: u64 = 500_000_000;
    let ix = make_sol_offer_ix(&buyer.pubkey(), &asset_kp.pubkey(), amount);
    assert_ok(send_ix(&mut svm, &[&buyer], ix));

    let (offer_pk, _) = offer_pda(&asset_kp.pubkey(), &buyer.pubkey());
    let offer = fetch_offer(&svm, &offer_pk);
    assert_eq!(offer.buyer, buyer.pubkey());
    assert_eq!(offer.asset, asset_kp.pubkey());
    assert_eq!(offer.amount, amount);
    assert_eq!(offer.payment_mint, None);

    let (vault_pk, _) = offer_vault_pda(&asset_kp.pubkey(), &buyer.pubkey());
    let balance = sol_balance(&svm, &vault_pk);
    assert_eq!(
        balance, amount,
        "vault should hold exactly the escrowed offer amount"
    );
}

#[test]
fn make_sol_offer_rejects_zero_amount() {
    let mut svm = init_svm();
    let _setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let buyer = funded_keypair(&mut svm);
    let asset_kp = Keypair::new();

    let ix = make_sol_offer_ix(&buyer.pubkey(), &asset_kp.pubkey(), 0);
    assert_marketplace_error(
        send_ix(&mut svm, &[&buyer], ix),
        MarketplaceError::InvalidPrice,
    );
}
