use std::collections::HashMap;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{SERVICE_ID, Service};
use currency::asset;
use currency::asset::{AssetId, MetaAsset, AssetBundle, AssetInfo};
use currency::wallet;
use currency::status;
use currency::error::Error;
use currency::transactions::components::Fees;

pub const ADD_ASSETS_ID: u16 = 300;

message!{
    struct AddAssets {
        const TYPE = SERVICE_ID;
        const ID = ADD_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey      [00 => 32]
        field meta_assets: Vec<MetaAsset> [32 => 40]
        field seed:        u64            [40 => 48]
    }
}

impl AddAssets {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_pub = Service::genesis_wallet();
        let creator_pub = self.pub_key();

        let mut genesis = wallet::Schema(&*view).fetch(&genesis_pub);
        let mut creator = wallet::Schema(&*view).fetch(&creator_pub);

        let fees = self.fees(view);

        wallet::move_coins(&mut creator, &mut genesis, fees.for_transaction())?;

        wallet::Schema(&mut*view).store(&genesis_pub, genesis.clone());
        wallet::Schema(&mut*view).store(&creator_pub, creator.clone());

        wallet::move_coins(&mut creator, &mut genesis, fees.for_assets_total())?;

        wallet::Schema(&mut*view).store(&genesis_pub, genesis);
        wallet::Schema(&mut*view).store(&creator_pub, creator);

        let assets = self.extract_assets(view)?;

        let mut recipients = HashMap::new();
        for (recipient, asset, info) in assets {
            let id = asset.id();
            recipients
                .entry(recipient)
                .or_insert(Vec::new())
                .push(asset);

            asset::Schema(&mut*view).store(&id, info);
        }

        for (key, assets) in recipients  {
            let mut recipient = wallet::Schema(&*view).fetch(&key);

            recipient.push_assets(assets);

            wallet::Schema(&mut*view).store(&key, recipient);
        }

        Ok(())
    }

    fn extract_assets(&self, view: &mut Fork) -> Result<Vec<(PublicKey, AssetBundle, AssetInfo)>, Error> {
        self.meta_assets().into_iter()
            .map(|meta| {
                let id = AssetId::from_data(meta.data(), &meta.receiver());
                let state = asset::Schema(&mut*view).fetch(&id);

                let key = self.pub_key();
                let info = state.map_or_else(
                    || Ok(meta.to_info(&key)),
                    |info| info.merge(meta.to_info(&key)),
                )?;

                let asset = meta.to_bundle(id);

                Ok((*meta.receiver(), asset, info))
            })
            .collect()
    }

    fn fees(&self, view: &mut Fork) -> Fees {
        Fees::new_add_assets(view, *self.pub_key(), self.meta_assets()).unwrap()
    }
}

impl Transaction for AddAssets {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return true;
        }

        if !self.verify_signature(&self.pub_key()) {
            return false;
        }
        
        for asset in self.meta_assets() {
            if !asset.verify() {
                return false;
            }
        }

        true
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}

