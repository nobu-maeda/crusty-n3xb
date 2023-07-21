use crate::interface::{nostr::*, *};
use crate::order::{Order, TradeEngineSpecfiicsTrait};
use crate::order_sm::maker::MakerSM;
use serde::Serialize;
use serde_json::{Map, Value};

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    interface: ArcInterface<EngineSpecificsType>,
    // order_cache: HashMap<Order>,
    // maker_sms: HashMap<MakerSM>,
    // taker_sms: HashMap<TakerSM>,
    trade_engine_name: String,
    _phantom_engine_specifics: PhantomData<EngineSpecificsType>,
}

impl<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize>
    Manager<EngineSpecificsType>
{
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

    fn load_settings() {
        // TODO: Read all files from relevant directories, scan for settings, and load into memory
        // Settings should be applied later as applicable from the memory location
    }

    pub async fn query_orders(&self) -> Vec<dyn Order> {
        let application_tag = OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string());

        let mut filter_tags = Map::new();
        filter_tags.insert(
            application_tag.key(),
            Value::Array(vec![Value::String(N3XB_APPLICATION_TAG.to_string())]),
        );

        let filter = Filter::new()
            .since(Timestamp::now())
            .kind(Kind::ParameterizedReplaceable(30078))
            .custom(filter_tags);

        let timeout = Duration::from_secs(1);

        let events = self
            .event_msg_client
            .lock()
            .unwrap()
            .get_events_of(vec![filter], Some(timeout))
            .await
            .unwrap();

        vec![]
    }

    // Private Functions
}
