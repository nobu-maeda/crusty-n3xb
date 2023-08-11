use tokio::sync::RwLock;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::common::error::N3xbError;
use crate::common::types::SerdeGenericTrait;
use crate::interface::{nostr::*, *};
use crate::offer::Offer;
use crate::order::Order;
use crate::order_sm::maker::{ArcMakerSM, MakerSM};
use crate::order_sm::taker::{ArcTakerSM, TakerSM};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager<
    OrderEngineSpecificType: SerdeGenericTrait,
    OfferEngineSpecificType: SerdeGenericTrait,
> {
    trade_engine_name: String,
    interface: ArcInterface<OrderEngineSpecificType, OfferEngineSpecificType>,
    order_cache: Vec<Order<OrderEngineSpecificType>>,
    maker_sms:
        RwLock<HashMap<String, ArcMakerSM<OrderEngineSpecificType, OfferEngineSpecificType>>>,
    taker_sms:
        RwLock<HashMap<String, ArcTakerSM<OrderEngineSpecificType, OfferEngineSpecificType>>>,
}

impl<OrderEngineSpecificType: SerdeGenericTrait, OfferEngineSpecificType: SerdeGenericTrait>
    Manager<OrderEngineSpecificType, OfferEngineSpecificType>
{
    // Public Functions

    // Constructors

    // TODO: Should take in genericized Keys or Client, but also Trade Engine Specifics
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(
        trade_engine_name: &str,
    ) -> Manager<OrderEngineSpecificType, OfferEngineSpecificType> {
        let nostr_interface = NostrInterface::new(trade_engine_name).await;
        Manager {
            trade_engine_name: trade_engine_name.to_string(),
            interface: Arc::new(Mutex::new(nostr_interface)),
            order_cache: Vec::new(),
            maker_sms: RwLock::new(HashMap::new()),
            taker_sms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_with_keys(
        keys: Keys,
        trade_engine_name: &str,
    ) -> Manager<OrderEngineSpecificType, OfferEngineSpecificType> {
        let nostr_interface = NostrInterface::new_with_keys(keys, trade_engine_name).await;
        Manager {
            trade_engine_name: trade_engine_name.to_string(),
            interface: Arc::new(Mutex::new(nostr_interface)),
            order_cache: Vec::new(),
            maker_sms: RwLock::new(HashMap::new()),
            taker_sms: RwLock::new(HashMap::new()),
        }
    }

    // Nostr Management
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

    pub async fn make_new_order(
        &self,
        order: Order<OrderEngineSpecificType>,
    ) -> Result<ArcMakerSM<OrderEngineSpecificType, OfferEngineSpecificType>, N3xbError> {
        let maker_sm: MakerSM<OrderEngineSpecificType, OfferEngineSpecificType> =
            MakerSM::new(Arc::clone(&self.interface), order.clone()).await?;
        let arc_maker_sm = Arc::new(Mutex::new(maker_sm));

        let mut maker_sms = self.maker_sms.write().await;
        maker_sms.insert(order.event_id.clone(), Arc::clone(&arc_maker_sm));
        Ok(arc_maker_sm)
    }

    pub async fn query_order_notes(
        &mut self,
    ) -> Result<Vec<Order<OrderEngineSpecificType>>, N3xbError> {
        let orders = self.interface.lock().unwrap().query_order_notes().await?;
        self.order_cache = orders.clone();
        Ok(orders)
    }

    pub async fn take_order(
        &self,
        order: Order<OrderEngineSpecificType>,
        offer: Offer<OfferEngineSpecificType>,
    ) -> Result<ArcTakerSM<OrderEngineSpecificType, OfferEngineSpecificType>, N3xbError> {
        let taker_sm: TakerSM<OrderEngineSpecificType, OfferEngineSpecificType> =
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
