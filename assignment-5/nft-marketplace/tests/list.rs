mod helpers;

use helpers::*;
use nft_marketplace::error::MarketplaceError;

#[test]
fn list_creates_listing_and_escrows_asset() {
    let mut svm = init_svm();
    let _setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let price: u64 = 1_000_000_000;
    let ix = list_ix(&maker.pubkey(), &asset.pubkey(), None, None, price);
    assert_ok(send_ix(&mut svm, &[&maker], ix));

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    let listing = fetch_listing(&svm, &listing_pk);
    assert_eq!(listing.maker, maker.pubkey());
    assert_eq!(listing.asset, asset.pubkey());
    assert_eq!(listing.price, price);
    assert_eq!(listing.payment_mint, None);

    let asset_data = fetch_asset(&svm, &asset.pubkey());
    assert_eq!(asset_data.owner, listing_pk);
}

#[test]
fn list_stores_payment_mint_when_provided() {
    let mut svm = init_svm();
    let setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);
    let payment_mint = create_spl_mint(
        &mut svm,
        &setup.admin,
        &setup.admin.pubkey(),
        PAYMENT_MINT_DECIMALS,
    );

    let price = PAYMENT_TOKEN_UNIT.checked_mul(100).unwrap();
    let ix = list_ix(
        &maker.pubkey(),
        &asset.pubkey(),
        None,
        Some(payment_mint.pubkey()),
        price,
    );
    assert_ok(send_ix(&mut svm, &[&maker], ix));

    let (listing_pk, _) = listing_pda(&asset.pubkey());
    let listing = fetch_listing(&svm, &listing_pk);
    assert_eq!(listing.payment_mint, Some(payment_mint.pubkey()));
}

#[test]
fn list_rejects_zero_price() {
    let mut svm = init_svm();
    let _setup = setup_marketplace(&mut svm, DEFAULT_FEE_BPS);
    let maker = funded_keypair(&mut svm);
    let asset = create_mpl_asset(&mut svm, &maker);

    let ix = list_ix(&maker.pubkey(), &asset.pubkey(), None, None, 0);
    assert_marketplace_error(
        send_ix(&mut svm, &[&maker], ix),
        MarketplaceError::InvalidPrice,
    );
}
