extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod evo_testkit;

use std::collections::HashMap;

use hyper::status::StatusCode;
use exonum::crypto;
use exonum_testkit::TestKit;
use evo_testkit::{EvoTestKit, EvoTestKitApi, asset_fees, create_asset};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::error::Error;

#[test]
fn fees_for_transfer() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    let tax = 10;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_key, _) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, amount, asset_fees(tax, 0), &creator_key);
    testkit.add_assets(&sender_pub_key, vec![asset.clone()], vec![info]);

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let (status, response) = api.post_fee(&tx_transfer);

    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + amount * tax;
    expected.insert(sender_pub_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_transfer_sender_is_creator() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    let tax = 10;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);

    testkit.set_configuration(Configuration::new(config_fees));

    let (recipient_key, _) = crypto::gen_keypair();
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, amount, asset_fees(tax, 0), &sender_pub_key);
    testkit.add_assets(&sender_pub_key, vec![asset.clone()], vec![info]);

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let (status, response) = api.post_fee(&tx_transfer);

    let mut expected = HashMap::new();
    expected.insert(sender_pub_key, transaction_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_transfer_asset_not_found() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    let tax = 10;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_key, _) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let (asset, _) = create_asset(meta_data, amount, asset_fees(tax, 0), &creator_key);

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let (status, response) = api.post_fee(&tx_transfer);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::AssetNotFound)));
}