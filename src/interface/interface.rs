use super::{maker_order_note::*, nostr::*};
use crate::order::*;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub struct NostrInterface<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    event_msg_client: ArcClient,
    subscription_client: ArcClient,
    _phantom_engine_specifics: PhantomData<EngineSpecificsType>,
}

impl<EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone + Serialize>
    NostrInterface<EngineSpecificsType>
{
    // Constructors
    pub async fn new() -> Self {
        let keys = Keys::generate();
        NostrInterface {
            event_msg_client: Self::new_nostr_client(&keys).await,
            subscription_client: Self::new_nostr_client(&keys).await,
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub async fn new_with_keys(keys: Keys) -> Self {
        NostrInterface {
            event_msg_client: Self::new_nostr_client(&keys).await,
            subscription_client: Self::new_nostr_client(&keys).await,
            _phantom_engine_specifics: PhantomData,
        }
    }

    pub fn new_with_nostr(event_msg_client: Client, subscription_client: Client) -> Self {
        NostrInterface {
            event_msg_client: Arc::new(Mutex::new(event_msg_client)),
            subscription_client: Arc::new(Mutex::new(subscription_client)),
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

    pub async fn send_event_note(&self, order: Order<EngineSpecificsType>) {
        // Create Note Content
        let maker_order_note = MakerOrderNote {
            maker_obligation: order.maker_obligation.content.to_owned(),
            taker_obligation: order.taker_obligation.content.to_owned(),
            trade_details: order.trade_details.content.to_owned(),
            trade_engine_specifics: order.engine_details.trade_engine_specifics.to_owned(),
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
        tag_set.push(OrderTag::TradeEngineName(
            order.engine_details.trade_engine_name.clone(),
        ));
        tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
        tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));

        // NIP-78 Event Kind - 30078
        let builder = EventBuilder::new(
            Kind::ParameterizedReplaceable(30078),
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

    fn create_event_tags(order_tags: Vec<OrderTag>) -> Vec<Tag> {
        order_tags
            .iter()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::*;

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

        let interface: NostrInterface<SomeTradeEngineSpecifics> =
            NostrInterface::new_with_nostr(event_msg_client, subscription_client);

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

        let engine_details = TradeEngineDetails {
            trade_engine_name: SomeTestParams::engine_name_str(),
            trade_engine_specifics: SomeTradeEngineSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            },
        };

        let order = Order {
            trade_uuid: SomeTestParams::some_uuid_string(),
            maker_obligation,
            taker_obligation,
            trade_details,
            engine_details,
            pow_difficulty: SomeTestParams::pow_difficulty(),
        };
    }
}
