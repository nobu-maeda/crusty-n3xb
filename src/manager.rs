use log::{debug, warn};
use std::collections::HashMap;
use std::net::SocketAddr;

use secp256k1::{SecretKey, XOnlyPublicKey};
use tokio::sync::RwLock;
use tokio::task::JoinError;
use url::Url;
use uuid::Uuid;

use crate::common::error::N3xbError;
use crate::communicator::{Communicator, CommunicatorAccess};
use crate::machine::maker::{Maker, MakerAccess};
use crate::machine::taker::{Taker, TakerAccess};
use crate::offer::Offer;
use crate::order::{FilterTag, Order, OrderEnvelope};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager {
    trade_engine_name: String,
    communicator: Communicator,
    communicator_accessor: CommunicatorAccess,
    makers: RwLock<HashMap<Uuid, Maker>>,
    takers: RwLock<HashMap<Uuid, Taker>>,
    maker_accessors: RwLock<HashMap<Uuid, MakerAccess>>,
    taker_accessors: RwLock<HashMap<Uuid, TakerAccess>>,
}

impl Manager {
    // Public Functions

    // Constructors

    // TODO: Should take in genericized Keys or Client, but also Trade Engine Specifics
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(trade_engine_name: impl AsRef<str>) -> Manager {
        let communicator = Communicator::new(trade_engine_name.as_ref()).await;
        let communicator_accessor = communicator.new_accessor();

        Manager {
            trade_engine_name: trade_engine_name.as_ref().to_string(),
            communicator,
            communicator_accessor,
            makers: RwLock::new(HashMap::new()),
            takers: RwLock::new(HashMap::new()),
            maker_accessors: RwLock::new(HashMap::new()),
            taker_accessors: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_with_keys(key: SecretKey, trade_engine_name: impl AsRef<str>) -> Manager {
        let communicator = Communicator::new_with_key(key, trade_engine_name.as_ref()).await;
        let communicator_accessor = communicator.new_accessor();

        Manager {
            trade_engine_name: trade_engine_name.as_ref().to_string(),
            communicator,
            communicator_accessor,
            makers: RwLock::new(HashMap::new()),
            takers: RwLock::new(HashMap::new()),
            maker_accessors: RwLock::new(HashMap::new()),
            taker_accessors: RwLock::new(HashMap::new()),
        }
    }

    // Nostr Management
    pub async fn pubkey(&self) -> XOnlyPublicKey {
        self.communicator_accessor.get_pubkey().await
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
        self.communicator_accessor
            .add_relays(relays, connect)
            .await?;
        Ok(())
    }

    pub async fn remove_relay(&self, relay: Url) -> Result<(), N3xbError> {
        debug!(
            "Manager w/ pubkey {} removing relay {:?}",
            self.pubkey().await,
            relay
        );
        self.communicator_accessor.remove_relay(relay).await?;
        Ok(())
    }

    pub async fn get_relays(&self) -> Vec<Url> {
        debug!("Manager w/ pubkey {} getting relays", self.pubkey().await);
        self.communicator_accessor.get_relays().await
    }

    // Order Management
    pub async fn new_maker(&self, order: Order) -> MakerAccess {
        let trade_uuid = order.trade_uuid;
        let maker = Maker::new(self.communicator.new_accessor(), order).await;
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
        let mut order_envelopes = self.communicator_accessor.query_orders(filter_tags).await?;
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
        let taker = Taker::new(self.communicator.new_accessor(), order_envelope, offer).await;
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

    fn load_settings() {
        // TODO: Read all files from relevant directories, scan for settings, and load into memory
        // Settings should be applied later as applicable from the memory location
    }

    pub async fn shutdown(self) -> Result<(), JoinError> {
        debug!("Manager w/ pubkey {} shutting down", self.pubkey().await);

        if let Some(error) = self.communicator_accessor.shutdown().await.err() {
            warn!("Manager error shutting down Communicator: {}", error);
        }
        self.communicator.task_handle.await?;
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
