use crate::interface::{nostr::*, *};
use crate::order::{OrderBuilder, TradeEngineSpecfiicsTrait};
use serde::Serialize;
use serde_json::{Map, Value};

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct Manager<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    interface: NostrInterface<EngineSpecificsType>,
    // order_cache: HashMap<Order>,
    // maker_sms: HashMap<MakerSM>,
    // taker_sms: HashMap<TakerSM>,

    // TODO: Local DB
    _phantom_engine_specifics: PhantomData<EngineSpecificsType>,
}

impl<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize>
    Manager<EngineSpecificsType>
{
    // Public Functions

    // Constructors

    pub async fn new() -> Self {
        Manager {
            interface: NostrInterface::new().await,
            // TODO: Create Local DB
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub async fn new_with_keys(keys: Keys) -> Self {
        Manager {
            interface: NostrInterface::new_with_keys(keys).await,
            // TODO: Create Local DB
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub fn new_with_nostr(event_msg_client: Client, subscription_client: Client) -> Self {
        Manager {
            interface: NostrInterface::new_with_nostr(event_msg_client, subscription_client),
            // TODO: Create Local DB
            _phantom_engine_specifics: PhantomData,
        }
    }

    // Order Management

    pub fn build_maker_order(&self) -> OrderBuilder<EngineSpecificsType> {
        OrderBuilder::new()
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
