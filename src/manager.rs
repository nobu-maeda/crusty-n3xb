use crate::common::types::SerdeGenericTrait;
use crate::error::N3xbError;
use crate::interface::{nostr::*, *};
use crate::order::Order;
use crate::order_sm::maker::MakerSM;
use crate::order_sm::taker::TakerSM;

use std::net::SocketAddr;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager<EngineSpecificsType: SerdeGenericTrait> {
    interface: ArcInterface<EngineSpecificsType>,
    // order_cache: HashMap<Order>,
    // maker_sms: HashMap<MakerSM>,
    // taker_sms: HashMap<TakerSM>,
    trade_engine_name: String,
    _phantom_engine_specifics: PhantomData<EngineSpecificsType>,
}

impl<EngineSpecificsType: SerdeGenericTrait> Manager<EngineSpecificsType> {
    // Public Functions

    // Constructors

    // TODO: Should take in genericized Keys or Client, but also Trade Engine Specifics
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(trade_engine_name: &str) -> Self {
        let nostr_interface = NostrInterface::new(trade_engine_name).await;
        Manager {
            interface: Arc::new(Mutex::new(nostr_interface)),
            trade_engine_name: trade_engine_name.to_string(),
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub async fn new_with_keys(keys: Keys, trade_engine_name: &str) -> Self {
        let nostr_interface = NostrInterface::new_with_keys(keys, trade_engine_name).await;
        Manager {
            interface: Arc::new(Mutex::new(nostr_interface)),
            trade_engine_name: trade_engine_name.to_string(),
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub fn new_with_nostr(
        event_msg_client: Client,
        subscription_client: Client,
        trade_engine_name: &str,
    ) -> Self {
        let nostr_interface = NostrInterface::new_with_nostr(
            event_msg_client,
            subscription_client,
            trade_engine_name,
        );
        Manager {
            interface: Arc::new(Mutex::new(nostr_interface)),
            trade_engine_name: trade_engine_name.to_string(),
            _phantom_engine_specifics: PhantomData,
        }
    }

    // Nostr Management
    pub async fn add_relays<S>(&self, relays: Vec<(S, u16, Option<SocketAddr>)>)
    where
        S: Into<String>,
    {
        self.interface.lock().unwrap().add_relays(relays).await;
    }

    // Order Management

    pub async fn make_new_order(
        &self,
        order: Order<EngineSpecificsType>,
    ) -> Result<MakerSM<EngineSpecificsType>, N3xbError> {
        //TODO: Persist MakerSM
        MakerSM::new(&self.interface, order).await
    }

    pub async fn query_order_notes(&self) -> Result<Vec<Order<EngineSpecificsType>>, N3xbError> {
        self.interface.lock().unwrap().query_order_notes().await
    }

    pub async fn take_order(
        &self,
        order: Order<EngineSpecificsType>,
    ) -> Result<TakerSM<EngineSpecificsType>, N3xbError> {
        //TODO: Persist TakerSM
        TakerSM::new(&self.interface, order).await
    }

    fn load_settings() {
        // TODO: Read all files from relevant directories, scan for settings, and load into memory
        // Settings should be applied later as applicable from the memory location
    }

    // Private Functions
}
