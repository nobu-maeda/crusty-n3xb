use super::{maker_order_note::*, nostr::*};
use crate::{
    error::N3xbError,
    order::{types::*, *},
};
use log::warn;
pub use serde_json::{Map, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{collections::HashSet, marker::PhantomData};

pub struct NostrInterface<EngineSpecificsType: TradeEngineSpecfiicsTrait> {
    event_msg_client: ArcClient,
    subscription_client: ArcClient,
    trade_engine_name: String,
    _phantom_engine_specifics: PhantomData<EngineSpecificsType>,
}

impl<EngineSpecificsType: TradeEngineSpecfiicsTrait> NostrInterface<EngineSpecificsType> {
    const MAKER_ORDER_NOTE_KIND: Kind = Kind::ParameterizedReplaceable(30078);

    // Constructors
    pub async fn new(trade_engine_name: &str) -> Self {
        let keys = Keys::generate();
        NostrInterface {
            event_msg_client: Self::new_nostr_client(&keys).await,
            subscription_client: Self::new_nostr_client(&keys).await,
            trade_engine_name: trade_engine_name.to_owned(),
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub async fn new_with_keys(keys: Keys, trade_engine_name: &str) -> Self {
        NostrInterface {
            event_msg_client: Self::new_nostr_client(&keys).await,
            subscription_client: Self::new_nostr_client(&keys).await,
            trade_engine_name: trade_engine_name.to_owned(),
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub fn new_with_nostr(
        event_msg_client: Client,
        subscription_client: Client,
        trade_engine_name: &str,
    ) -> Self {
        NostrInterface {
            event_msg_client: Arc::new(Mutex::new(event_msg_client)),
            subscription_client: Arc::new(Mutex::new(subscription_client)),
            trade_engine_name: trade_engine_name.to_owned(),
            _phantom_engine_specifics: PhantomData,
        }
    }

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

    // Send Maker Order

    pub async fn send_maker_order_note(&self, order: Order<EngineSpecificsType>) {
        // Create Note Content
        let maker_order_note = MakerOrderNote {
            maker_obligation: order.maker_obligation.content.to_owned(),
            taker_obligation: order.taker_obligation.content.to_owned(),
            trade_details: order.trade_details.content.to_owned(),
            trade_engine_specifics: order.trade_engine_specifics.to_owned(),
            pow_difficulty: order.pow_difficulty,
        };

        let content_string = serde_json::to_string(&maker_order_note).unwrap(); // TODO: Error Handling?

        // Create Note Tags
        let mut tag_set: Vec<OrderTag> = Vec::new();

        tag_set.push(OrderTag::TradeUUID(order.trade_uuid.clone()));
        tag_set.push(OrderTag::MakerObligations(
            order.maker_obligation.kind.to_tags(),
        ));
        tag_set.push(OrderTag::TakerObligations(
            order.taker_obligation.kind.to_tags(),
        ));
        tag_set.push(OrderTag::TradeDetailParameters(
            order.trade_details.parameters_to_tags(),
        ));
        tag_set.push(OrderTag::TradeEngineName(self.trade_engine_name.to_owned()));
        tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
        tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));

        // NIP-78 Event Kind - 30078
        let builder = EventBuilder::new(
            Self::MAKER_ORDER_NOTE_KIND,
            content_string,
            &Self::create_event_tags(tag_set),
        );

        let keys = self.event_msg_client.lock().unwrap().keys();
        self.event_msg_client
            .lock()
            .unwrap()
            .send_event(builder.to_event(&keys).unwrap())
            .await
            .unwrap();
    }

    fn create_event_tags(tags: Vec<OrderTag>) -> Vec<Tag> {
        tags.iter()
            .map(|event_tag| match event_tag {
                OrderTag::TradeUUID(trade_uuid_string) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    vec![trade_uuid_string.to_owned()],
                ),
                OrderTag::MakerObligations(obligations) => Tag::Generic(
                    TagKind::Custom(event_tag.key()),
                    obligations.to_owned().into_iter().collect(),
                ),
                OrderTag::TakerObligations(obligations) => Tag::Generic(
                    TagKind::Custom(event_tag.key()),
                    obligations.to_owned().into_iter().collect(),
                ),
                OrderTag::TradeDetailParameters(parameters) => Tag::Generic(
                    TagKind::Custom(event_tag.key()),
                    parameters.to_owned().into_iter().collect(),
                ),
                OrderTag::TradeEngineName(name) => {
                    Tag::Generic(TagKind::Custom(event_tag.key()), vec![name.to_owned()])
                }
                OrderTag::EventKind(kind) => {
                    Tag::Generic(TagKind::Custom(event_tag.key()), vec![kind.to_string()])
                }
                OrderTag::ApplicationTag(app_tag) => {
                    Tag::Generic(TagKind::Custom(event_tag.key()), vec![app_tag.to_owned()])
                }
            })
            .collect()
    }

    pub async fn query_order_notes(&self) -> Result<Vec<Order<EngineSpecificsType>>, N3xbError> {
        let mut tag_set: Vec<OrderTag> = Vec::new();
        tag_set.push(OrderTag::TradeEngineName(self.trade_engine_name.to_owned()));
        tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
        tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));

        let filter = Self::create_event_tag_filter(tag_set);
        let timeout = Duration::from_secs(1);
        let events = self
            .subscription_client
            .lock()
            .unwrap()
            .get_events_of(vec![filter], Some(timeout))
            .await?;

        let maybe_orders = self.extract_orders_from_events(events);
        let mut orders: Vec<Order<EngineSpecificsType>> = Vec::new();
        for maybe_order in maybe_orders {
            match maybe_order {
                Ok(order) => orders.push(order),
                Err(error) => {
                    warn!(
                        "Order extraction from Nostr event failed - {}",
                        error.to_string()
                    );
                }
            }
        }
        Ok(orders)
    }

    fn extract_order_tags_from_tags(&self, tags: Vec<Tag>) -> Vec<OrderTag> {
        let mut order_tags: Vec<OrderTag> = Vec::new();
        for tag in tags {
            if let Tag::Generic(kind, value) = tag {
                if let TagKind::Custom(key) = kind {
                    if let Ok(order_tag) = OrderTag::from_key(key.clone(), value) {
                        order_tags.push(order_tag);
                    } else {
                        warn!("Unrecognized Tag with key: {}", key);
                    }
                } else {
                    warn!("Unexpected Tag with kind: {}", kind.to_string());
                }
            } else {
                warn!("Unexpected Tag extracted");
            }
        }
        order_tags
    }

    fn extract_order_from_event(
        &self,
        event: Event,
    ) -> Result<Order<EngineSpecificsType>, N3xbError> {
        let maker_order_note: MakerOrderNote<EngineSpecificsType> =
            serde_json::from_str(event.content.as_str())?;
        let order_tags = self.extract_order_tags_from_tags(event.tags);

        let mut some_trade_uuid: Option<String> = None;
        let mut some_maker_obligation_kind: Option<ObligationKind> = None;
        let mut some_taker_obligation_kind: Option<ObligationKind> = None;
        let mut trade_parameters: HashSet<TradeParameter> = HashSet::new();

        for order_tag in order_tags {
            match order_tag {
                OrderTag::TradeUUID(trade_uuid) => some_trade_uuid = Some(trade_uuid),
                OrderTag::MakerObligations(obligations) => {
                    some_maker_obligation_kind = Some(ObligationKind::from_tags(obligations)?);
                }
                OrderTag::TakerObligations(obligations) => {
                    some_taker_obligation_kind = Some(ObligationKind::from_tags(obligations)?);
                }
                OrderTag::TradeDetailParameters(parameters) => {
                    trade_parameters = TradeDetails::tags_to_parameters(parameters);
                }

                // Sanity Checks. Abort order parsing if fails
                OrderTag::TradeEngineName(name) => {
                    if name != self.trade_engine_name {
                        let message = format!("Trade Engine Name {} mismatch on Maker Order Note deserialization. {} expected.", name, self.trade_engine_name);
                        warn!("{}", message);
                        return Err(N3xbError::Simple(message));
                    }
                }
                OrderTag::EventKind(event_kind) => {
                    if event_kind != EventKind::MakerOrder {
                        let message = format!("Trade Engine Name {} mismatch on Maker Order Note deserialization. {} expected.", event_kind.to_string(), EventKind::MakerOrder.to_string());
                        warn!("{}", message);
                        return Err(N3xbError::Simple(message));
                    }
                }
                OrderTag::ApplicationTag(app_tag) => {
                    if app_tag != N3XB_APPLICATION_TAG {
                        let message = format!("Application Tag {} mismatch on Maker Order Note deserialization. {} expected.", app_tag, N3XB_APPLICATION_TAG);
                        warn!("{}", message);
                        return Err(N3xbError::Simple(message));
                    }
                }
            }
        }

        let maker_obligation = if let Some(obligation_kind) = some_maker_obligation_kind {
            MakerObligation {
                kind: obligation_kind,
                content: maker_order_note.maker_obligation,
            }
        } else {
            let message = format!("Invalid or missing Maker Obligation Kind in Maker Order Note");
            warn!("{}", message);
            return Err(N3xbError::Simple(message));
        };

        let taker_obligation = if let Some(obligation_kind) = some_taker_obligation_kind {
            TakerObligation {
                kind: obligation_kind,
                content: maker_order_note.taker_obligation,
            }
        } else {
            let message = format!("Invalid or missing Taker Obligation Kind in Maker Order Note");
            warn!("{}", message);
            return Err(N3xbError::Simple(message));
        };

        let trade_details = TradeDetails {
            parameters: trade_parameters,
            content: maker_order_note.trade_details,
        };

        let trade_uuid = if let Some(uuid) = some_trade_uuid {
            uuid
        } else {
            let message = format!("Invalid or missing Trade UUID in Maker Order Note");
            warn!("{}", message);
            return Err(N3xbError::Simple(message));
        };

        Ok(Order {
            trade_uuid,
            maker_obligation,
            taker_obligation,
            trade_details,
            trade_engine_specifics: maker_order_note.trade_engine_specifics,
            pow_difficulty: maker_order_note.pow_difficulty,
        })
    }

    fn extract_orders_from_events(
        &self,
        events: Vec<Event>,
    ) -> Vec<Result<Order<EngineSpecificsType>, N3xbError>> {
        let mut orders: Vec<Result<Order<EngineSpecificsType>, N3xbError>> = Vec::new();
        for event in events {
            let order = self.extract_order_from_event(event);
            orders.push(order);
        }
        orders
    }

    fn create_event_tag_filter(tags: Vec<OrderTag>) -> Filter {
        let mut tag_map = Map::new();
        tags.iter().for_each(|tag| match tag {
            OrderTag::TradeUUID(trade_uuid_string) => {
                tag_map.insert(tag.key(), Value::String(trade_uuid_string.to_owned()));
            }
            OrderTag::MakerObligations(obligations) => {
                tag_map.insert(tag.key(), obligations.to_owned().into_iter().collect());
            }
            OrderTag::TakerObligations(obligations) => {
                tag_map.insert(tag.key(), obligations.to_owned().into_iter().collect());
            }
            OrderTag::TradeDetailParameters(parameters) => {
                tag_map.insert(tag.key(), parameters.to_owned().into_iter().collect());
            }
            OrderTag::TradeEngineName(name) => {
                tag_map.insert(
                    tag.key(),
                    Value::Array(vec![Value::String(name.to_owned())]),
                );
            }
            OrderTag::EventKind(kind) => {
                tag_map.insert(
                    tag.key(),
                    Value::Array(vec![Value::String(kind.to_string())]),
                );
            }
            OrderTag::ApplicationTag(app_tag) => {
                tag_map.insert(
                    tag.key(),
                    Value::Array(vec![Value::String(app_tag.to_owned())]),
                );
            }
        });

        Filter::new()
            .kind(Self::MAKER_ORDER_NOTE_KIND)
            .custom(tag_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::testing::*;

    fn send_event_expectation(event: Event) -> Result<EventId, Error> {
        print!("Nostr Event: {:?}", event); // TODO: Actually validate the event, Tags and JSON content is as expected
        Result::Ok(event.id)
    }

    #[tokio::test]
    async fn order_send_maker_note() {
        let mut event_msg_client = Client::new();
        event_msg_client
            .expect_keys()
            .returning(|| Keys::generate());
        event_msg_client
            .expect_send_event()
            .returning(send_event_expectation);

        let subscription_client = Client::new();

        let interface: NostrInterface<SomeTradeEngineMakerOrderSpecifics> =
            NostrInterface::new_with_nostr(
                event_msg_client,
                subscription_client,
                &SomeTestParams::engine_name_str(),
            );

        let maker_obligation = MakerObligation {
            kind: SomeTestParams::maker_obligation_kind(),
            content: SomeTestParams::maker_obligation_content(),
        };

        let taker_obligation = TakerObligation {
            kind: SomeTestParams::taker_obligation_kind(),
            content: SomeTestParams::taker_obligation_content(),
        };

        let trade_details = TradeDetails {
            parameters: SomeTestParams::trade_parameters(),
            content: SomeTestParams::trade_details_content(),
        };

        let trade_engine_specifics = SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        };

        let order = Order {
            trade_uuid: SomeTestParams::some_uuid_string(),
            maker_obligation,
            taker_obligation,
            trade_details,
            trade_engine_specifics,
            pow_difficulty: SomeTestParams::pow_difficulty(),
        };
    }
}
