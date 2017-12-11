use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::storage::StorageValue;

use service;
use service::wallet::Asset;
use service::transaction::add_assets::TxAddAsset;
use service::transaction::create_wallet::TxCreateWallet;
use service::transaction::del_assets::TxDelAsset;
use service::transaction::exchange::{TxExchange, ExchangeOffer};
use service::transaction::mining::TxMining;
use service::transaction::trade_assets::{TxTrade, TradeOffer};
use service::transaction::transfer::TxTransfer;

pub struct TransactionBuilder {
    public_key: Option<PublicKey>,
    secret_key: Option<SecretKey>,
    network_id: u32,
    protocol_version: u32,
    service_id: u16,
}

struct TransactionMetadata {
    public_key: PublicKey,
    secret_key: SecretKey,
    network_id: u32,
    protocol_version: u32,
    service_id: u16,
}

impl From<TransactionBuilder> for TransactionMetadata {
    fn from(b: TransactionBuilder) -> Self {
        TransactionMetadata {
            public_key: b.public_key.unwrap(),
            secret_key: b.secret_key.unwrap(),
            network_id: b.network_id,
            protocol_version: b.protocol_version,
            service_id: b.service_id,
        }
    }
}

impl TransactionBuilder {
    pub fn new() -> Self {
        TransactionBuilder {
            public_key: None,
            secret_key: None,
            network_id: 0,
            protocol_version: 0,
            service_id: service::SERVICE_ID,
        }
    }

    pub fn keypair(self, public_key: PublicKey, secret_key: SecretKey) -> Self {
        TransactionBuilder {
            public_key: Some(public_key),
            secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn random_keypair(self) -> Self {
        let (public_key, secret_key) = crypto::gen_keypair();
        TransactionBuilder {
            public_key: Some(public_key),
            secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn network_id(self, network_id: u32) -> Self {
        TransactionBuilder { network_id, ..self }
    }

    pub fn protocol_version(self, protocol_version: u32) -> Self {
        TransactionBuilder { protocol_version, ..self }
    }

    pub fn service_id(self, service_id: u16) -> Self {
        TransactionBuilder { service_id, ..self }
    }

    pub fn tx_add_assets(self) -> TxAddAssetBuilder {
        self.validate();
        TxAddAssetBuilder::new(self.into())
    }

    pub fn tx_create_wallet(self) -> TxCreateWalletBuilder {
        self.validate();
        TxCreateWalletBuilder::new(self.into())
    }

    pub fn tx_del_assets(self) -> TxDelAssetBuilder {
        self.validate();
        TxDelAssetBuilder::new(self.into())
    }

    pub fn tx_exchange(self) -> TxExchangeBuilder {
        self.validate();
        TxExchangeBuilder::new(self.into())
    }

    pub fn tx_mining(self) -> TxMiningBuilder {
        self.validate();
        TxMiningBuilder::new(self.into())
    }

    pub fn tx_trade_assets(self) -> TxTradeBuilder {
        self.validate();
        TxTradeBuilder::new(self.into())
    }

    pub fn tx_transfer(self) -> TxTransferBuilder {
        self.validate();
        TxTransferBuilder::new(self.into())
    }

    fn validate(&self) {
        match (&self.public_key, &self.secret_key) {
            (&Some(_), &Some(_)) => (),
            _ => panic!("Public and secret keys must be set."),
        }
    }
}

pub struct TxAddAssetBuilder {
    meta: TransactionMetadata,
    assets: Vec<Asset>,
    seed: u64,
}

impl TxAddAssetBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxAddAssetBuilder {
            meta,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn add_asset(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        TxAddAssetBuilder { seed, ..self }
    }

    pub fn build(self) -> TxAddAsset {
        self.validate();
        TxAddAsset::new(
            &self.meta.public_key,
            self.assets,
            self.seed,
            &self.meta.secret_key,
        )
    }

    fn validate(&self) {
    }
}

pub struct TxCreateWalletBuilder {
    meta: TransactionMetadata,
}

impl TxCreateWalletBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxCreateWalletBuilder {
            meta,
        }
    }

    pub fn build(self) -> TxCreateWallet {
        self.validate();
        TxCreateWallet::new(
            &self.meta.public_key,
            &self.meta.secret_key,
        )
    }

    fn validate(&self) {
    }
}

pub struct TxDelAssetBuilder {
    meta: TransactionMetadata,
    assets: Vec<Asset>,
    seed: u64,
}

impl TxDelAssetBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxDelAssetBuilder {
            meta,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn add_asset(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        TxDelAssetBuilder { seed, ..self }
    }

    pub fn build(self) -> TxDelAsset {
        self.validate();
        TxDelAsset::new(
            &self.meta.public_key,
            self.assets,
            self.seed,
            &self.meta.secret_key,
        )
    }

    fn validate(&self) {
    }
}

pub struct TxExchangeBuilder {
    meta: TransactionMetadata,

    sender_assets: Vec<Asset>,
    sender_value: u64,

    recipient: Option<PublicKey>,
    recipient_assets: Vec<Asset>,
    recipient_value: u64,

    fee_strategy: u8,

    seed: u64,
}

impl TxExchangeBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxExchangeBuilder {
            meta,

            sender_assets: Vec::new(),
            sender_value: 0,

            recipient: None,
            recipient_assets: Vec::new(),
            recipient_value: 0,

            fee_strategy: 1,

            seed: 0,
        }
    }

    pub fn sender_add_asset(mut self, asset: Asset) -> Self {
        self.sender_assets.push(asset);
        self
    }

    pub fn sender_value(self, sender_value: u64) -> Self {
        TxExchangeBuilder {
            sender_value,
            ..self
        }
    }

    pub fn recipient(self, pub_key: PublicKey) -> Self {
        TxExchangeBuilder {
            recipient: Some(pub_key),
            ..self
        }
    }

    pub fn recipient_add_asset(mut self, asset: Asset) -> Self {
        self.recipient_assets.push(asset);
        self
    }

    pub fn recipient_value(self, recipient_value: u64) -> Self {
        TxExchangeBuilder {
            recipient_value,
            ..self
        }
    }

    pub fn fee_strategy(self, fee_strategy: u8) -> Self {
        TxExchangeBuilder {
            fee_strategy,
            ..self
        }
    }

    pub fn seed(self, seed: u64) -> Self {
        TxExchangeBuilder {
            seed,
            ..self
        }
    }

    pub fn build(self) -> TxExchange {
        self.verify();
        let offer = ExchangeOffer::new(
            &self.meta.public_key,
            self.sender_assets,
            self.sender_value,
            self.recipient.as_ref().unwrap(),
            self.recipient_assets,
            self.recipient_value,
            self.fee_strategy,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.meta.secret_key);
        TxExchange::new(
            offer,
            self.seed,
            &signature,
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.recipient.is_some());
    }
}

pub struct TxMiningBuilder {
    meta: TransactionMetadata,
    seed: u64,
}

impl TxMiningBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxMiningBuilder {
            meta,
            seed: 0,
        }
    }

    pub fn seed(self, seed: u64) -> Self {
        TxMiningBuilder {
            seed,
            ..self
        }
    }

    pub fn build(self) -> TxMining {
        self.verify();
        TxMining::new(
            &self.meta.public_key,
            self.seed,
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
    }
}

pub struct TxTradeBuilder {
    meta: TransactionMetadata,
    buyer: Option<PublicKey>,
    assets: Vec<Asset>,
    price: u64,
    seed: u64,
}

impl TxTradeBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxTradeBuilder {
            meta,
            buyer: None,
            assets: Vec::new(),
            price: 0,
            seed: 0,
        }
    }

    pub fn buyer(self, pub_key: PublicKey) -> Self {
        TxTradeBuilder {
            buyer: Some(pub_key),
            ..self
        }
    }

    pub fn add_asset(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn price(self, price: u64) -> Self {
        TxTradeBuilder {
            price,
            ..self
        }
    }

    pub fn build(self) -> TxTrade {
        self.verify();

        let offer = TradeOffer::new(
                &self.meta.public_key,
                self.assets,
                self.price,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.meta.secret_key);
        TxTrade::new(
            self.buyer.as_ref().unwrap(),
            offer,
            self.seed,
            &signature,
            &self.meta.secret_key
        )
    }

    fn verify(&self) {
        assert!(self.buyer.is_some());
    }
}

pub struct TxTransferBuilder {
    meta: TransactionMetadata,
    recipient: Option<PublicKey>,
    amount: u64,
    assets: Vec<Asset>,
    seed: u64,
}

impl TxTransferBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TxTransferBuilder {
            meta,
            recipient: None,
            amount: 0,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn recipient(self, pub_key: PublicKey) -> Self {
        TxTransferBuilder {
            recipient: Some(pub_key),
            ..self
        }
    }

    pub fn amount(self, amount: u64) -> Self {
        TxTransferBuilder {
            amount,
            ..self
        }
    }

    pub fn add_asset(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        TxTransferBuilder {
            seed,
            ..self
        }
    }

    pub fn build(self) -> TxTransfer {
        self.verify();

        TxTransfer::new(
            &self.meta.public_key,
            self.recipient.as_ref().unwrap(),
            self.amount,
            self.assets,
            self.seed,
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.recipient.is_some());
    }
}

#[cfg(test)]
mod test {
    use exonum::crypto;

    use service::wallet::Asset;

    use service::transaction::add_assets::TxAddAsset;

    // TODO: tests.
    // use service::transaction::create_wallet::TxCreateWallet;
    // use service::transaction::del_assets::TxDelAsset;
    // use service::transaction::exchange::{TxExchange, ExchangeOffer};
    // use service::transaction::mining::TxMining;
    // use service::transaction::trade_assets::{TxTrade, TradeOffer};
    // use service::transaction::transfer::TxTransfer;

    use test::transaction::TransactionBuilder;

    #[test]
    #[should_panic]
    fn meta_underspecified() {
        TransactionBuilder::new().tx_add_assets();
    }

    #[test]
    fn add_assets() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let transaction = TransactionBuilder::new()
            .random_keypair()
            .tx_add_assets()
            .add_asset(Asset::new("foobar", 9))
            .add_asset(Asset::new("bazqux", 13))
            .build();

        let assets = vec![Asset::new("foobar", 9), Asset::new("bazqux", 13)];
        let equivalent = TxAddAsset::new(&public_key, assets, 0, &secret_key);

        assert!(transaction == equivalent);
    }
}

