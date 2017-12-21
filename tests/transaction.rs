extern crate exonum;
extern crate dmbc;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::storage::{Database, MemoryDB};
use exonum::messages::Message;

use dmbc::service::asset::{Asset, AssetID, AssetInfo};
use dmbc::service::builders::transaction;
use dmbc::service::builders::wallet;
use dmbc::service::schema::asset::AssetSchema;
use dmbc::service::schema::wallet::WalletSchema;
use dmbc::service::schema::transaction_status::{TxStatusSchema, TxStatus};

#[test]
fn add_assets() {
    let (public_key, secret_key) = crypto::gen_keypair();

    let absent_data = "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f";
    let existing_data = "a8d5c97d-9978-4111-9947-7a95dcb31d0f";

    let absent_id = AssetID::new(absent_data, &public_key).unwrap();
    let existing_id = AssetID::new(existing_data, &public_key).unwrap();

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset(absent_data, 45)
        .add_asset(existing_data, 17)
        .seed(85)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| {
        s.assets().put(&existing_id, AssetInfo::new(&public_key, 3))
    });

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(2000)
        .add_asset(existing_data, 3)
        .build();

    WalletSchema::map(fork, |mut s| s.wallets().put(&public_key, wallet));

    tx.execute(fork);

    let existing_info = AssetSchema::map(fork, |mut s| s.info(&existing_id).unwrap());

    assert_eq!(20, existing_info.amount());

    let wallet = WalletSchema::map(fork, |mut s| s.wallet(tx.pub_key()).unwrap());

    assert_eq!(2000 - tx.get_fee(), wallet.balance());
    assert_eq!(20, wallet.asset(existing_id).unwrap().amount());
    assert_eq!(45, wallet.asset(absent_id).unwrap().amount());
}

#[test]
fn create_wallet() {
    use dmbc::service::transaction::INIT_BALANCE;

    let (public_key, secret_key) = crypto::gen_keypair();
    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_create_wallet()
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(INIT_BALANCE)
        .build();

    tx.execute(fork);

    WalletSchema::map(fork, |mut schema| {
        assert_eq!(Some(wallet), schema.wallet(tx.pub_key()));
    });
}

#[test]
fn delete_assets() {
    let (public_key, secret_key) = crypto::gen_keypair();

    let data_1 = "deleted";
    let data_2 = "removed from wallet";
    let data_3 = "removed from network";

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset(data_1, 10)
        .add_asset(data_2, 20)
        .add_asset(data_3, 30)
        .seed(113)
        .build();

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(2000)
        .add_asset(data_1, 20)
        .add_asset(data_2, 20)
        .add_asset(data_3, 30)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    let id_1 = AssetID::new(data_1, &public_key).unwrap();
    let id_2 = AssetID::new(data_2, &public_key).unwrap();
    let id_3 = AssetID::new(data_3, &public_key).unwrap();

    AssetSchema::map(fork, |mut s| {
        s.add_asset(&id_1, &public_key, 30);
        s.add_asset(&id_2, &public_key, 30);
        s.add_asset(&id_3, &public_key, 30);
    });

    WalletSchema::map(fork, move |mut s| s.wallets().put(&public_key, wallet));

    tx.execute(fork);

    AssetSchema::map(fork, |mut s| {
        assert_eq!(Some(20), s.info(&id_1).map(|a| a.amount()));
        assert_eq!(Some(10), s.info(&id_2).map(|a| a.amount()));
        assert_eq!(None, s.info(&id_3).map(|a| a.amount()));
    });

    WalletSchema::map(fork, |mut s| {
        let wallet = s.wallet(&public_key).unwrap();
        assert_eq!(Some(Asset::new(id_1, 10)), wallet.asset(id_1));
        assert_eq!(None, wallet.asset(id_2));
        assert_eq!(None, wallet.asset(id_3));
    });
}

#[test]
fn delete_assets_fails() {
    let (public_key, secret_key) = crypto::gen_keypair();

    let data = "asset";
    let id = AssetID::new(data, &public_key).unwrap();

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(2000)
        .add_asset(data, 20)
        .build();

    let tx_too_many = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset(data, 30)
        .seed(9)
        .build();

    let tx_doesnt_exist = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset("absent", 999)
        .seed(9)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| s.add_asset(&id, &public_key, 20));
    WalletSchema::map(fork, |mut s| s.wallets().put(&public_key, wallet));

    tx_too_many.execute(fork);

    AssetSchema::map(fork, |mut s| {
        assert_eq!(Some(20), s.info(&id).map(|a| a.amount()));
    });

    WalletSchema::map(fork, |mut s| {
        assert_eq!(
            Some(20),
            s.wallet(&public_key)
             .and_then(|w| w.asset(id))
             .map(|a| a.amount()));
    });

    tx_doesnt_exist.execute(fork);

    TxStatusSchema::map(fork, |mut s| {
        assert_eq!(Some(TxStatus::Fail), s.get_status(&tx_doesnt_exist.hash()));
    });
}

#[test]
fn exchange() {
    let (sender_public, sender_secret) = crypto::gen_keypair();
    let (recipient_public, _) = crypto::gen_keypair();

    let sender_data_1 = "sender asset 1";
    let sender_id_1 = AssetID::new(sender_data_1, &sender_public).unwrap();

    let sender_data_2 = "sender asset 2";
    let sender_id_2 = AssetID::new(sender_data_2, &sender_public).unwrap();

    let recipient_data_1 = "recipient asset 1";
    let recipient_id_1 = AssetID::new(recipient_data_1, &recipient_public).unwrap();

    let recipient_data_2 = "recipient asset 2";
    let recipient_id_2 = AssetID::new(recipient_data_2, &recipient_public).unwrap();

    let sender = wallet::Builder::new()
        .key(sender_public)
        .balance(100)
        .add_asset(sender_data_1, 10)
        .add_asset(sender_data_2, 30)
        .build();

    let recipient = wallet::Builder::new()
        .key(recipient_public)
        .balance(100)
        .add_asset(recipient_data_1, 30)
        .add_asset(recipient_data_2, 50)
        .build();

    let tx = transaction::Builder::new()
        .keypair(sender_public, sender_secret.clone())
        .tx_exchange()
        .sender_add_asset(sender_data_1, 10)
        .sender_add_asset(sender_data_2, 15)
        .sender_value(50)
        .recipient(recipient_public)
        .recipient_add_asset(recipient_data_1, 30)
        .recipient_add_asset(recipient_data_2, 25)
        .recipient_value(0)
        .fee_strategy(1)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| {
        s.add_asset(&sender_id_1, &sender_public, 10);
        s.add_asset(&sender_id_2, &sender_public, 30);
        s.add_asset(&recipient_id_1, &recipient_public, 30);
        s.add_asset(&recipient_id_2, &recipient_public, 50);
    });

    WalletSchema::map(fork, |mut s| {
        s.wallets().put(&sender_public, sender);
        s.wallets().put(&recipient_public, recipient);
    });

    tx.execute(fork);

    WalletSchema::map(fork, |mut s| {
        let sender = s.wallet(&sender_public).unwrap();
        let recipient = s.wallet(&recipient_public).unwrap();

        assert_eq!(None, sender.asset(sender_id_1).map(|a| a.amount()));
        assert_eq!(Some(15), sender.asset(sender_id_2).map(|a| a.amount()));
        assert_eq!(Some(30), sender.asset(recipient_id_1).map(|a| a.amount()));
        assert_eq!(Some(25), sender.asset(recipient_id_2).map(|a| a.amount()));

        assert_eq!(None, recipient.asset(recipient_id_1).map(|a| a.amount()));
        assert_eq!(Some(25), recipient.asset(recipient_id_2).map(|a| a.amount()));
        assert_eq!(Some(10), recipient.asset(sender_id_1).map(|a| a.amount()));
        assert_eq!(Some(15), recipient.asset(sender_id_2).map(|a| a.amount()));
    });
}

#[test]
fn trade_assets() {
    let (seller_public, seller_secret) = crypto::gen_keypair();
    let (buyer_public, _) = crypto::gen_keypair();

    let full_data = "fully transferred asset";
    let full_id = AssetID::new(full_data, &seller_public).unwrap();

    let half_data = "partially transferred asset";
    let half_id = AssetID::new(half_data, &seller_public).unwrap();

    let seller = wallet::Builder::new()
        .key(seller_public)
        .balance(2000)
        .add_asset(full_data, 20)
        .add_asset(half_data, 20)
        .build();

    let buyer = wallet::Builder::new()
        .key(buyer_public)
        .balance(2000)
        .build();

    let tx = transaction::Builder::new()
        .keypair(seller_public, seller_secret)
        .tx_trade_assets()
        .buyer(buyer_public)
        .add_asset(full_data, 20)
        .add_asset(half_data, 10)
        .price(88)
        .seed(4)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| {
        s.assets().put(&full_id, AssetInfo::new(&seller_public, 20));
        s.assets().put(&half_id, AssetInfo::new(&seller_public, 20));
    });

    WalletSchema::map(fork, |mut s| {
        s.wallets().put(&seller_public, seller);
        s.wallets().put(&buyer_public, buyer);
    });

    tx.execute(fork);

    WalletSchema::map(fork, |mut s| {
        let seller = s.wallet(&seller_public).unwrap();
        let buyer = s.wallet(&buyer_public).unwrap();

        assert_eq!(None,     seller.asset(full_id).map(|a| a.amount()));
        assert_eq!(Some(10), seller.asset(half_id).map(|a| a.amount()));

        assert_eq!(Some(20), buyer.asset(full_id).map(|a| a.amount()));
        assert_eq!(Some(10), buyer.asset(half_id).map(|a| a.amount()));

        assert_eq!(2000 - tx.get_fee() + 88, seller.balance());
        assert_eq!(2000 - 88, buyer.balance());
    });
}

#[test]
fn transfer() {
    let (sender_public, sender_secret) = crypto::gen_keypair();
    let (recipient_public, _) = crypto::gen_keypair();

    let full_data = "fully transferred asset";
    let full_id = AssetID::new(full_data, &sender_public).unwrap();

    let half_data = "partially transferred asset";
    let half_id = AssetID::new(half_data, &sender_public).unwrap();

    let sender = wallet::Builder::new()
        .key(sender_public)
        .balance(2000)
        .add_asset(full_data, 20)
        .add_asset(half_data, 20)
        .build();

    let recipient = wallet::Builder::new()
        .key(recipient_public)
        .balance(2000)
        .build();

    let tx = transaction::Builder::new()
        .keypair(sender_public, sender_secret)
        .tx_transfer()
        .recipient(recipient_public)
        .amount(100)
        .add_asset(full_data, 20)
        .add_asset(half_data, 10)
        .seed(123)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| {
        s.assets().put(&full_id, AssetInfo::new(&sender_public, 20));
        s.assets().put(&half_id, AssetInfo::new(&sender_public, 20));
    });

    WalletSchema::map(fork, |mut s| {
        s.wallets().put(&sender_public, sender);
        s.wallets().put(&recipient_public, recipient);
    });

    tx.execute(fork);

    WalletSchema::map(fork, |mut s| {
        let sender = s.wallet(&sender_public).unwrap();
        let recipient = s.wallet(&recipient_public).unwrap();

        assert_eq!(None,     sender.asset(full_id).map(|a| a.amount()));
        assert_eq!(Some(10), sender.asset(half_id).map(|a| a.amount()));

        assert_eq!(Some(20), recipient.asset(full_id).map(|a| a.amount()));
        assert_eq!(Some(10), recipient.asset(half_id).map(|a| a.amount()));

        assert_eq!(2000 - tx.get_fee() - 100, sender.balance());
        assert_eq!(2000 + 100, recipient.balance());
    });
}
