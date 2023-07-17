use crate::nostr::*;
use crate::order::{OrderBuilder, TradeEngineSpecfiicsTrait};
use serde::Serialize;
use serde_json::{Map, Value};

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct Manager<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    event_msg_client: ArcClient,
    subscription_client: ArcClient,

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

    pub async fn new_with_keys(keys: Keys) -> Self {
        Manager {
            event_msg_client: Self::new_nostr_client(&keys).await,
            subscription_client: Self::new_nostr_client(&keys).await,
            // TODO: Create Local DB
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub async fn new() -> Self {
        let keys = Keys::generate();

        Manager {
            event_msg_client: Self::new_nostr_client(&keys).await,
            subscription_client: Self::new_nostr_client(&keys).await,
            // TODO: Create Local DB
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub fn new_with_nostr(event_msg_client: Client, subscription_client: Client) -> Self {
        Manager {
            event_msg_client: Arc::new(Mutex::new(event_msg_client)),
            subscription_client: Arc::new(Mutex::new(subscription_client)),
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

    async fn new_nostr_client(keys: &Keys) -> ArcClient {
        let opts = Options::new()
            .wait_for_connection(true)
            .wait_for_send(true)
            .difficulty(8);
        let client = Client::with_opts(&keys, opts);

        client.add_relay("ws://localhost:8008", None).await.unwrap(); // TODO: Should add to existing list of relay, or default relay list, vs localhost test mode?
        client.connect().await;
        Arc::new(Mutex::new(client))
    }
}
