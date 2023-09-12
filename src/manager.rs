use tokio::sync::RwLock;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::common::error::N3xbError;
use crate::interface::{nostr::*, *};
use crate::offer::Offer;
use crate::order::Order;
use crate::order_sm::maker::{ArcMakerSM, MakerSM};
use crate::order_sm::taker::{ArcTakerSM, TakerSM};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager {
    trade_engine_name: String,
    interface: ArcInterface,
    order_cache: Vec<Order>,
    maker_sms: RwLock<HashMap<String, ArcMakerSM>>,
    taker_sms: RwLock<HashMap<String, ArcTakerSM>>,
}

impl Manager {
    // Public Functions

    // Constructors

    // TODO: Should take in genericized Keys or Client, but also Trade Engine Specifics
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(trade_engine_name: &str) -> Manager {
        let interface = NostrInterface::new(trade_engine_name).await;

        Manager {
            trade_engine_name: trade_engine_name.to_string(),
            interface: Arc::new(Mutex::new(interface)),
            order_cache: Vec::new(),
            maker_sms: RwLock::new(HashMap::new()),
            taker_sms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_with_keys(keys: Keys, trade_engine_name: &str) -> Manager {
        let interface = NostrInterface::new_with_keys(keys, trade_engine_name).await;

        Manager {
            trade_engine_name: trade_engine_name.to_string(),
            interface: Arc::new(Mutex::new(interface)),
            order_cache: Vec::new(),
            maker_sms: RwLock::new(HashMap::new()),
            taker_sms: RwLock::new(HashMap::new()),
        }
    }

    // Nostr Management
    pub async fn pubkey(&self) -> String {
        let interface = self.interface.lock().unwrap();
        interface.pubkey().await
    }

    pub async fn add_relays<S>(&self, relays: Vec<(S, Option<SocketAddr>)>, connect: bool)
    where
        S: Into<String> + 'static,
    {
        self.interface
            .lock()
            .unwrap()
            .add_relays(relays, connect)
            .await;
    }

    // Order Management

    pub async fn make_new_order(&self, order: Order) -> Result<ArcMakerSM, N3xbError> {
        let maker_sm: MakerSM = MakerSM::new(Arc::clone(&self.interface), order.clone()).await?;
        let arc_maker_sm = Arc::new(Mutex::new(maker_sm));

        let mut maker_sms = self.maker_sms.write().await;
        maker_sms.insert(order.event_id.clone(), Arc::clone(&arc_maker_sm));
        Ok(arc_maker_sm)
    }

    pub async fn query_order_notes(&mut self) -> Result<Vec<Order>, N3xbError> {
        let orders = self.interface.lock().unwrap().query_order_notes().await?;
        self.order_cache = orders.clone();
        Ok(orders)
    }

    pub async fn take_order(&self, order: Order, offer: Offer) -> Result<ArcTakerSM, N3xbError> {
        let taker_sm: TakerSM =
            TakerSM::new(Arc::clone(&self.interface), order.clone(), offer).await?;
        let arc_taker_sm = Arc::new(Mutex::new(taker_sm));

        let mut taker_sms = self.taker_sms.write().await;
        taker_sms.insert(order.event_id.clone(), Arc::clone(&arc_taker_sm));
        Ok(arc_taker_sm)
    }

    fn load_settings() {
        // TODO: Read all files from relevant directories, scan for settings, and load into memory
        // Settings should be applied later as applicable from the memory location
    }

    // Private Functions
}
