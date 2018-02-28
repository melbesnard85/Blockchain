use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::AssetBundle;
use currency::transactions::components::Intermediary;

pub const EXCHANGE_INTERMEDIARY_ID: u16 = 602;

encoding_struct! {
    struct ExchangeOfferIntermediary {
        const SIZE = 97;

        field intermediary:     Intermediary     [00 =>  8]

        field sender:           &PublicKey       [08 => 40]
        field sender_assets:    Vec<AssetBundle> [40 => 48]
        field sender_value:     u64              [48 => 56]

        field recipient:        &PublicKey       [56 => 88]
        field recipient_assets: Vec<AssetBundle> [88 => 96]

        field fee_strategy:     u8               [96 => 97]
    }
}

message! {
    struct ExchangeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:                  ExchangeOfferIntermediary [0 => 8]
        field seed:                   u64                       [8 => 16]
        field sender_signature:       &Signature                [16 => 80]
        field intermediary_signature: &Signature                [80 => 144]
        field data_info:              &str                      [144 => 152]
    }
}

impl ExchangeIntermediary {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }
}

impl Transaction for ExchangeIntermediary {
    fn verify(&self) -> bool {
        unimplemented!()
    }

    fn execute(&self, view: &mut Fork) {
        let _ = view;
        unimplemented!()
    }

    fn info(&self) -> serde_json::Value {
        unimplemented!()
    }
}
