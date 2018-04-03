use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;
use prometheus::Counter;

use currency::{Service, SERVICE_ID};
use currency::assets::AssetBundle;
use currency::transactions::components::{FeeStrategy, ThirdPartyFees};
use currency::error::Error;
use currency::status;
use currency::wallet;
use currency::configuration::Configuration;

/// Transaction ID.
pub const EXCHANGE_ID: u16 = 601;

encoding_struct! {
    struct ExchangeOffer {
        const SIZE = 89;

        field sender:           &PublicKey       [00 => 32]
        field sender_assets:    Vec<AssetBundle> [32 => 40]
        field sender_value:     u64              [40 => 48]

        field recipient:        &PublicKey       [48 => 80]
        field recipient_assets: Vec<AssetBundle> [80 => 88]

        field fee_strategy:     u8               [88 => 89]
    }
}

message! {
    /// `exchange` transaction.
    struct Exchange {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_ID;
        const SIZE = 88;

        field offer:             ExchangeOffer     [00 => 8]
        field seed:              u64               [8 => 16]
        field sender_signature:  &Signature        [16 => 80]
        field data_info:         &str              [80 => 88]
    }
}

impl Exchange {
    /// Get raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_fee = Configuration::extract(view).fees().exchange();

        let offer = self.offer();

        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        match fee_strategy {
            FeeStrategy::Recipient => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());

                wallet::move_coins(&mut recipient, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Sender => {
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                wallet::move_coins(&mut sender, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.sender(), sender);
            }
            FeeStrategy::RecipientAndSender => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                wallet::move_coins(&mut recipient, &mut genesis, genesis_fee / 2)?;
                wallet::move_coins(&mut sender, &mut genesis, genesis_fee / 2)?;

                wallet::Schema(&mut *view).store(offer.sender(), sender);
                wallet::Schema(&mut *view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Intermediary => return Err(Error::InvalidTransaction),
        }

        wallet::Schema(&mut *view).store(&Service::genesis_wallet(), genesis);

        let fees = ThirdPartyFees::new_exchange(
            &*view,
            offer
                .sender_assets()
                .into_iter()
                .chain(offer.recipient_assets().into_iter()),
        )?;

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = match fee_strategy {
            FeeStrategy::Recipient => fees.collect(view, offer.recipient())?,
            FeeStrategy::Sender => fees.collect(view, offer.sender())?,
            FeeStrategy::RecipientAndSender => fees.collect2(view, offer.sender(), offer.recipient())?,
            FeeStrategy::Intermediary => unreachable!(),
        };

        // Process the main transaction.
        let mut sender = updated_wallets
            .remove(&offer.sender())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.sender()));
        let mut recipient = updated_wallets
            .remove(&offer.recipient())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.recipient()));

        wallet::move_coins(&mut sender, &mut recipient, offer.sender_value())?;
        wallet::move_assets(&mut sender, &mut recipient, &offer.sender_assets())?;
        wallet::move_assets(&mut recipient, &mut sender, &offer.recipient_assets())?;

        updated_wallets.insert(*offer.sender(), sender);
        updated_wallets.insert(*offer.recipient(), recipient);

        // Save changes to the database.
        for (key, wallet) in updated_wallets {
            wallet::Schema(&mut *view).store(&key, wallet);
        }

        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
}

impl Transaction for Exchange {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let offer = self.offer();

        let wallets_ok = offer.sender() != offer.recipient();
        let fee_strategy_ok = match FeeStrategy::try_from(offer.fee_strategy()).unwrap() {
            FeeStrategy::Recipient | FeeStrategy::Sender | FeeStrategy::RecipientAndSender => true,
            _ => false,
        };

        if cfg!(fuzzing) {
            return wallets_ok && fee_strategy_ok;
        }

        let recipient_ok = self.verify_signature(offer.recipient());
        let sender_ok = crypto::verify(self.sender_signature(), &offer.raw, offer.sender());

        if wallets_ok && fee_strategy_ok && recipient_ok && sender_ok {
            VERIFY_SUCCESS_COUNT.inc();
            true
        } else {
            false
        }

    }

    fn execute(&self, view: &mut Fork) {
        EXECUTE_COUNT.inc();

        let result = self.process(view);

        if let &Ok(_) = &result {
            EXECUTE_SUCCESS_COUNT.inc();
        }

        status::Schema(view).store(self.hash(), result);

        EXECUTE_FINISH_COUNT.inc();
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
