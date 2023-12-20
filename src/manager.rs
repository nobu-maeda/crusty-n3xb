use log::{debug, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

use secp256k1::{SecretKey, XOnlyPublicKey};
use tokio::sync::RwLock;
use tokio::task::JoinError;
use url::Url;
use uuid::Uuid;

use crate::common::error::N3xbError;
use crate::comms::{Comms, CommsAccess};
use crate::maker::{Maker, MakerAccess};
use crate::offer::Offer;
use crate::order::{FilterTag, Order, OrderEnvelope};
use crate::taker::{Taker, TakerAccess};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager {
    trade_engine_name: String,
    pubkey: XOnlyPublicKey,
    comms: Comms,
    comms_accessor: CommsAccess,
    makers: RwLock<HashMap<Uuid, Maker>>,
    takers: RwLock<HashMap<Uuid, Taker>>,
    maker_accessors: RwLock<HashMap<Uuid, MakerAccess>>,
    taker_accessors: RwLock<HashMap<Uuid, TakerAccess>>,
}

impl Manager {
    // Constructors
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(trade_engine_name: impl AsRef<str>) -> Manager {
        let comms = Comms::new(trade_engine_name.as_ref()).await;
        Self::new_with_comms(comms, trade_engine_name).await
    }

    pub async fn new_with_key(key: SecretKey, trade_engine_name: impl AsRef<str>) -> Manager {
        let comms = Comms::new_with_key(key, trade_engine_name.as_ref()).await;
        Self::new_with_comms(comms, trade_engine_name).await
    }

    async fn new_with_comms(comms: Comms, trade_engine_name: impl AsRef<str>) -> Manager {
        let comms_accessor = comms.new_accessor();
        let pubkey = comms_accessor.get_pubkey().await;

        let (makers, takers) = Self::setup_and_restore(&comms_accessor, pubkey.to_string()).await;
        let mut maker_accessors = HashMap::new();
        for maker in &makers {
            maker_accessors.insert(maker.0.clone(), maker.1.new_accessor().await);
        }
        let mut taker_accessors = HashMap::new();
        for taker in &takers {
            taker_accessors.insert(taker.0.clone(), taker.1.new_accessor().await);
        }

        Manager {
            trade_engine_name: trade_engine_name.as_ref().to_string(),
            pubkey,
            comms,
            comms_accessor,
            makers: RwLock::new(makers),
            takers: RwLock::new(takers),
            maker_accessors: RwLock::new(maker_accessors),
            taker_accessors: RwLock::new(taker_accessors),
        }
    }

    fn maker_data_dir_path(identifier: impl AsRef<str>) -> String {
        format!("data/{}/makers", identifier.as_ref())
    }

    fn taker_data_dir_path(identifier: impl AsRef<str>) -> String {
        format!("data/{}/takers", identifier.as_ref())
    }

    async fn setup_and_restore(
        comms_accessor: &CommsAccess,
        identifier: impl AsRef<str>,
    ) -> (HashMap<Uuid, Maker>, HashMap<Uuid, Taker>) {
        let result: Result<(HashMap<Uuid, Maker>, HashMap<Uuid, Taker>), N3xbError> = async {
            // Create directories to data and manager with identifier if not already exist
            let maker_dir_path = Self::maker_data_dir_path(&identifier);
            tokio::fs::create_dir_all(&maker_dir_path).await?;

            // Restore Makers from files in maker directory
            let makers = Self::restore_makers(comms_accessor, &maker_dir_path).await;

            // Do the same for Takers
            let taker_dir_path = Self::taker_data_dir_path(&identifier);
            tokio::fs::create_dir_all(&taker_dir_path).await?;

            let takers = Self::restore_takers(comms_accessor, &taker_dir_path).await?;
            Ok((makers, takers))
        }
        .await;

        match result {
            Ok((makers, takers)) => {
                debug!(
                    "Manager w/ pubkey {} restored {} Makers and {} Takers",
                    identifier.as_ref(),
                    makers.len(),
                    takers.len()
                );
                (makers, takers)
            }
            Err(err) => {
                warn!("Error setting up & restoring from data directory - {}", err);
                (HashMap::new(), HashMap::new())
            }
        }
    }

    async fn restore_makers(
        comms_accessor: &CommsAccess,
        maker_dir_path: impl AsRef<Path>,
    ) -> HashMap<Uuid, Maker> {
        // Go through all files in maker directory and restore each file as a new Maker
        let mut makers = HashMap::new();
        let mut maker_files = tokio::fs::read_dir(maker_dir_path).await.unwrap();
        while let Some(maker_file) = maker_files.next_entry().await.unwrap() {
            let maker_file_path = maker_file.path();
            let (trade_uuid, maker) =
                match Maker::restore(comms_accessor.clone(), &maker_file_path).await {
                    Ok((trade_uuid, maker)) => (trade_uuid, maker),
                    Err(err) => {
                        warn!(
                            "Error restoring Maker from file {:?} - {}",
                            maker_file_path, err
                        );
                        continue;
                    }
                };
            makers.insert(trade_uuid, maker);
        }
        makers
    }

    async fn restore_takers(
        comms_accessor: &CommsAccess,
        taker_dir_path: impl AsRef<Path>,
    ) -> Result<HashMap<Uuid, Taker>, N3xbError> {
        // Go through all files in taker directory and restore each file as a new Taker
        let mut takers = HashMap::new();
        let mut taker_files = tokio::fs::read_dir(taker_dir_path).await?;
        while let Some(taker_file) = taker_files.next_entry().await? {
            let taker_file_path = taker_file.path();
            let (trade_uuid, taker) =
                match Taker::restore(comms_accessor.clone(), &taker_file_path).await {
                    Ok((trade_uuid, taker)) => (trade_uuid, taker),
                    Err(err) => {
                        warn!(
                            "Error restoring Taker from file {:?} - {}",
                            taker_file_path, err
                        );
                        continue;
                    }
                };
            takers.insert(trade_uuid, taker);
        }
        Ok(takers)
    }

    // Nostr Management
    pub async fn pubkey(&self) -> XOnlyPublicKey {
        self.comms_accessor.get_pubkey().await
    }

    pub async fn add_relays(
        &self,
        relays: Vec<(Url, Option<SocketAddr>)>,
        connect: bool,
    ) -> Result<(), N3xbError> {
        debug!(
            "Manager w/ pubkey {} adding relays {:?}",
            self.pubkey().await,
            relays
        );
        self.comms_accessor.add_relays(relays, connect).await?;
        Ok(())
    }

    pub async fn remove_relay(&self, relay: Url) -> Result<(), N3xbError> {
        debug!(
            "Manager w/ pubkey {} removing relay {:?}",
            self.pubkey().await,
            relay
        );
        self.comms_accessor.remove_relay(relay).await?;
        Ok(())
    }

    pub async fn get_relays(&self) -> Vec<Url> {
        debug!("Manager w/ pubkey {} getting relays", self.pubkey().await);
        self.comms_accessor.get_relays().await
    }

    pub async fn connect_relay(&self, relay: Url) -> Result<(), N3xbError> {
        debug!(
            "Manager w/ pubkey {} connecting relay {:?}",
            self.pubkey().await,
            relay
        );
        self.comms_accessor.connect_relay(relay).await?;
        Ok(())
    }

    pub async fn connect_all_relays(&self) -> Result<(), N3xbError> {
        debug!(
            "Manager w/ pubkey {} connecting all relays",
            self.pubkey().await
        );
        self.comms_accessor.connect_all_relays().await?;
        Ok(())
    }

    // Order Management
    pub async fn new_maker(&self, order: Order) -> MakerAccess {
        let trade_uuid = order.trade_uuid;
        let maker = Maker::new(
            self.comms.new_accessor(),
            order,
            Self::maker_data_dir_path(self.pubkey.to_string()),
        )
        .await;
        let maker_my_accessor = maker.new_accessor().await;
        let maker_returned_accessor = maker.new_accessor().await;

        debug!(
            "Manager w/ pubkey {} adding Maker w/ TradeUUID {}",
            self.pubkey().await,
            trade_uuid
        );

        let mut makers = self.makers.write().await;
        makers.insert(trade_uuid, maker);

        let mut maker_accessors = self.maker_accessors.write().await;
        maker_accessors.insert(trade_uuid, maker_my_accessor);

        maker_returned_accessor
    }

    pub async fn query_orders(
        &self,
        filter_tags: Vec<FilterTag>,
    ) -> Result<Vec<OrderEnvelope>, N3xbError> {
        let mut order_envelopes = self.comms_accessor.query_orders(filter_tags).await?;
        let queried_length = order_envelopes.len();

        let valid_order_envelopes: Vec<OrderEnvelope> = order_envelopes
            .drain(..)
            .filter(|order_envelope| order_envelope.order.validate().is_ok())
            .collect();
        let valid_length = valid_order_envelopes.len();

        debug!(
            "Manager w/ pubkey {} queried {} orders and found {} valid orders",
            self.pubkey().await,
            queried_length,
            valid_length
        );

        if valid_length < queried_length {
            let filtered_orders = queried_length - valid_length;
            warn!("{} orders filtered out on original query result of {} orders leaving {} valid orders returned", filtered_orders, queried_length, valid_length);
        }
        Ok(valid_order_envelopes)
    }

    pub async fn new_taker(
        &self,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Result<TakerAccess, N3xbError> {
        offer.validate_against(&order_envelope.order)?;

        let trade_uuid = order_envelope.order.trade_uuid;
        let taker = Taker::new(
            self.comms.new_accessor(),
            order_envelope,
            offer,
            Self::taker_data_dir_path(self.pubkey.to_string()),
        )
        .await;
        let taker_my_accessor = taker.new_accessor().await;
        let taker_returned_accessor = taker.new_accessor().await;

        debug!(
            "Manager w/ pubkey {} adding Taker w/ TradeUUID {}",
            self.pubkey().await,
            trade_uuid
        );

        let mut takers = self.takers.write().await;
        takers.insert(trade_uuid, taker);

        let mut taker_accessors = self.taker_accessors.write().await;
        taker_accessors.insert(trade_uuid, taker_my_accessor);

        Ok(taker_returned_accessor)
    }

    pub async fn get_makers(&self) -> Vec<(Uuid, MakerAccess)> {
        let mut maker_accessors = self.maker_accessors.read().await.clone();
        maker_accessors
            .drain()
            .map(|(uuid, maker_accessor)| (uuid, maker_accessor))
            .collect()
    }

    pub async fn get_takers(&self) -> Vec<(Uuid, TakerAccess)> {
        let mut taker_accessors = self.taker_accessors.read().await.clone();
        taker_accessors
            .drain()
            .map(|(uuid, taker_accessor)| (uuid, taker_accessor))
            .collect()
    }

    pub async fn shutdown(self) -> Result<(), JoinError> {
        debug!("Manager w/ pubkey {} shutting down", self.pubkey().await);

        if let Some(error) = self.comms_accessor.shutdown().await.err() {
            warn!("Manager error shutting down Comms: {}", error);
        }
        self.comms.task_handle.await?;
        let mut makers = self.makers.write().await;
        for (_uuid, maker) in makers.drain() {
            maker.task_handle.await?;
        }
        let mut takers = self.takers.write().await;
        for (_uuid, taker) in takers.drain() {
            taker.task_handle.await?;
        }
        Ok(())
    }
}
