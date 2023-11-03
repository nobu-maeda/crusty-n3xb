use std::collections::HashMap;
use std::net::SocketAddr;

use secp256k1::{SecretKey, XOnlyPublicKey};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::N3xbError;
use crate::interfacer::{Interfacer, InterfacerHandle};
use crate::offer::Offer;
use crate::order::Order;
use crate::order_sm::maker::{Maker, MakerEngine};
use crate::order_sm::taker::{Taker, TakerEngine};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager {
    trade_engine_name: String,
    interfacer: Interfacer,
    interfacer_handle: InterfacerHandle,
    maker_engines: RwLock<HashMap<Uuid, MakerEngine>>,
    taker_engines: RwLock<HashMap<Uuid, TakerEngine>>,
    makers: RwLock<HashMap<Uuid, Maker>>,
    takers: RwLock<HashMap<Uuid, Taker>>,
}

impl Manager {
    // Public Functions

    // Constructors

    // TODO: Should take in genericized Keys or Client, but also Trade Engine Specifics
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(trade_engine_name: impl AsRef<str>) -> Manager {
        let interfacer = Interfacer::new(trade_engine_name.as_ref()).await;
        let interfacer_handle = interfacer.new_handle();

        Manager {
            trade_engine_name: trade_engine_name.as_ref().to_string(),
            interfacer,
            interfacer_handle,
            maker_engines: RwLock::new(HashMap::new()),
            taker_engines: RwLock::new(HashMap::new()),
            makers: RwLock::new(HashMap::new()),
            takers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_with_keys(key: SecretKey, trade_engine_name: impl AsRef<str>) -> Manager {
        let interfacer = Interfacer::new_with_key(key, trade_engine_name.as_ref()).await;
        let interfacer_handle = interfacer.new_handle();

        Manager {
            trade_engine_name: trade_engine_name.as_ref().to_string(),
            interfacer,
            interfacer_handle,
            maker_engines: RwLock::new(HashMap::new()),
            taker_engines: RwLock::new(HashMap::new()),
            makers: RwLock::new(HashMap::new()),
            takers: RwLock::new(HashMap::new()),
        }
    }

    // Nostr Management
    pub async fn pubkey(&self) -> XOnlyPublicKey {
        self.interfacer_handle.get_public_key().await
    }

    pub async fn add_relays(
        &self,
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
    ) -> Result<(), N3xbError> {
        self.interfacer_handle.add_relays(relays, connect).await?;
        Ok(())
    }

    // Order Management

    pub async fn make_new_order(&self, order: Order) -> Result<Maker, N3xbError> {
        let trade_uuid = order.trade_uuid;
        let maker_engine = MakerEngine::new(self.interfacer.new_handle(), order).await;
        let maker_own = maker_engine.new_handle().await;
        let maker_returned = maker_engine.new_handle().await;

        let mut maker_engines = self.maker_engines.write().await;
        maker_engines.insert(trade_uuid, maker_engine);

        let mut makers = self.makers.write().await;
        makers.insert(trade_uuid, maker_own);

        Ok(maker_returned)
    }

    pub async fn query_order_notes(&mut self) -> Result<Vec<Order>, N3xbError> {
        let orders = self.interfacer_handle.query_order_notes().await?;

        // TODO: Validate received Orders

        Ok(orders)
    }

    pub async fn take_order(&self, order: Order, offer: Offer) -> Result<Taker, N3xbError> {
        let trade_uuid = order.trade_uuid;
        let taker_engine = TakerEngine::new(self.interfacer.new_handle(), order, offer).await;
        let taker_own = taker_engine.new_handle().await;
        let taker_returned = taker_engine.new_handle().await;

        let mut taker_engines = self.taker_engines.write().await;
        taker_engines.insert(trade_uuid, taker_engine);

        let mut takers = self.takers.write().await;
        takers.insert(trade_uuid, taker_own);

        Ok(taker_returned)
    }

    fn load_settings() {
        // TODO: Read all files from relevant directories, scan for settings, and load into memory
        // Settings should be applied later as applicable from the memory location
    }

    // Private Functions
}
