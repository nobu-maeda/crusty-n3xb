pub use serde_json::{Map, Value};

use log::warn;
use std::str::FromStr;
use std::time::Duration;
use std::{collections::HashSet, marker::PhantomData};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::{maker_order_note::*, nostr::*, peer_messaging::*, poller::*};
use crate::common::error::N3xbError;
use crate::common::types::*;
use crate::offer::Offer;
use crate::order::*;

struct Router<OfferEngineSpecificType> {
    _phantom_order_specifics: PhantomData<OfferEngineSpecificType>,
}

impl<OfferEngineSpecificType: SerdeGenericTrait> Router<OfferEngineSpecificType> {
    fn new() -> Self {
        Router {
            _phantom_order_specifics: PhantomData,
        }
    }

    fn offer_callback(&mut self, _peer_message: PeerMessage<Offer<OfferEngineSpecificType>>) -> () {
    }
}

type ArcRouter<T> = Arc<std::sync::Mutex<Router<T>>>;

pub struct NostrInterface<
    OrderEngineSpecificType: SerdeGenericTrait,
    OfferEngineSpecificType: SerdeGenericTrait,
> {
    arc_client: ArcClient,
    poller_handle: JoinHandle<()>,
    trade_engine_name: String,
    arc_router: ArcRouter<OfferEngineSpecificType>,
    _phantom_order_specifics: PhantomData<OrderEngineSpecificType>,
    _phantom_offer_specifics: PhantomData<OfferEngineSpecificType>,
}

impl<OrderEngineSpecificType: SerdeGenericTrait, OfferEngineSpecificType: SerdeGenericTrait>
    NostrInterface<OrderEngineSpecificType, OfferEngineSpecificType>
{
    const MAKER_ORDER_NOTE_KIND: Kind = Kind::ParameterizedReplaceable(30078);

    // Constructors
    pub async fn new(trade_engine_name: &str) -> Self {
        Self::new_with_keys(Keys::generate(), trade_engine_name).await
    }

    pub async fn new_with_keys(keys: Keys, trade_engine_name: &str) -> Self {
        let arc_client = Self::new_nostr_client(&keys).await;
        Self::new_with_nostr(arc_client, trade_engine_name).await
    }

    pub async fn new_with_nostr(arc_client: ArcClient, trade_engine_name: &str) -> Self {
        let router = Router::new();
        let arc_router = Arc::new(std::sync::Mutex::new(router));
        let arc_router_clone = arc_router.clone();

        let event_config = EventConfig {
            offer_callback: Box::new(move |peer_message| {
                arc_router_clone
                    .lock()
                    .unwrap()
                    .offer_callback(peer_message)
            }),
        };
        let arc_event_config = Arc::new(tokio::sync::Mutex::new(event_config));

        let interface = NostrInterface {
            arc_client: Arc::clone(&arc_client),
            poller_handle: Poller::<OfferEngineSpecificType>::start(
                Arc::clone(&arc_client),
                arc_event_config,
            ),
            trade_engine_name: trade_engine_name.to_owned(),
            arc_router: arc_router.clone(),
            _phantom_order_specifics: PhantomData,
            _phantom_offer_specifics: PhantomData,
        };
        let client = arc_client.lock().await;
        let pubkey = client.keys().public_key();
        client
            .subscribe(interface.subscription_filters(pubkey))
            .await;
        interface
    }

    async fn new_nostr_client(keys: &Keys) -> ArcClient {
        let opts = Options::new()
            .wait_for_connection(true)
            .wait_for_send(true)
            .difficulty(8);
        let client = Client::with_opts(&keys, opts);
        // TODO: Add saved or default clients
        client.connect().await;
        Arc::new(Mutex::new(client))
    }

    // Nostr Client Management

    pub async fn pubkey(&self) -> String {
        let client = self.arc_client.lock().await;
        client.keys().public_key().to_string()
    }

    pub async fn add_relays<S>(&self, relays: Vec<(S, Option<SocketAddr>)>, connect: bool)
    where
        S: Into<String> + 'static,
    {
        let client = self.arc_client.lock().await;
        client.add_relays(relays).await.unwrap();
        if connect {
            let pubkey = client.keys().public_key();
            client.subscribe(self.subscription_filters(pubkey)).await;
            client.connect().await;
        }
    }

    fn subscription_filters(&self, pubkey: XOnlyPublicKey) -> Vec<Filter> {
        // Need a way to track existing Filters
        // Need a way to correlate State Machines to Subscriptions as to remove filters as necessary

        // Subscribe to all DM to own pubkey. Filter unrecognized DM out some other way. Can be spam prone
        let dm_filter = Filter::new().since(Timestamp::now()).pubkey(pubkey);

        vec![dm_filter]
    }

    // Add/Start Orders Subscription Filter

    // Start Peer Message Subscription

    // Add/Start

    // Send Maker Order Note

    pub async fn send_maker_order_note(
        &self,
        order: Order<OrderEngineSpecificType>,
    ) -> Result<(), N3xbError> {
        // Create Note Content
        let maker_order_note = MakerOrderNote {
            maker_obligation: order.maker_obligation.content.to_owned(),
            taker_obligation: order.taker_obligation.content.to_owned(),
            trade_details: order.trade_details.content.to_owned(),
            trade_engine_specifics: order.trade_engine_specifics.to_owned(),
            pow_difficulty: order.pow_difficulty,
        };

        let content_string = serde_json::to_string(&maker_order_note)?;

        // Create Note Tags
        let mut tag_set: Vec<OrderTag> = Vec::new();

        tag_set.push(OrderTag::TradeUUID(order.trade_uuid.clone()));
        tag_set.push(OrderTag::MakerObligations(
            order
                .maker_obligation
                .kinds
                .iter()
                .flat_map(|k| k.to_tags())
                .collect(),
        ));
        tag_set.push(OrderTag::TakerObligations(
            order
                .taker_obligation
                .kinds
                .iter()
                .flat_map(|k| k.to_tags())
                .collect(),
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

        let client = self.arc_client.lock().await;
        let keys = client.keys();
        client.send_event(builder.to_event(&keys).unwrap()).await?;
        Ok(())
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

    // Query Order Notes

    pub async fn query_order_notes(
        &self,
    ) -> Result<Vec<Order<OrderEngineSpecificType>>, N3xbError> {
        let mut tag_set: Vec<OrderTag> = Vec::new();
        tag_set.push(OrderTag::TradeEngineName(self.trade_engine_name.to_owned()));
        tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
        tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));

        let filter = Self::create_event_tag_filter(tag_set);
        let timeout = Duration::from_secs(1);
        let client = self.arc_client.lock().await;
        let events = client.get_events_of(vec![filter], Some(timeout)).await?;

        let maybe_orders = self.extract_orders_from_events(events);
        let mut orders: Vec<Order<OrderEngineSpecificType>> = Vec::new();
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
            let mut tag_vec = tag.as_vec();
            let tag_key = tag_vec.remove(0);

            if let Ok(order_tag) = OrderTag::from_key(tag_key.clone(), tag_vec) {
                order_tags.push(order_tag);
            } else {
                warn!("Unrecognized Tag with key: {}", tag_key);
            }
        }
        order_tags
    }

    fn extract_order_from_event(
        &self,
        event: Event,
    ) -> Result<Order<OrderEngineSpecificType>, N3xbError> {
        let maker_order_note: MakerOrderNote<OrderEngineSpecificType> =
            serde_json::from_str(event.content.as_str())?;
        let order_tags = self.extract_order_tags_from_tags(event.tags);

        let mut some_trade_uuid: Option<String> = None;
        let mut some_maker_obligation_kinds: Option<HashSet<ObligationKind>> = None;
        let mut some_taker_obligation_kinds: Option<HashSet<ObligationKind>> = None;
        let mut trade_parameters: HashSet<TradeParameter> = HashSet::new();

        for order_tag in order_tags {
            match order_tag {
                OrderTag::TradeUUID(trade_uuid) => some_trade_uuid = Some(trade_uuid),
                OrderTag::MakerObligations(obligations) => {
                    some_maker_obligation_kinds = Some(ObligationKind::from_tags(obligations)?);
                }
                OrderTag::TakerObligations(obligations) => {
                    some_taker_obligation_kinds = Some(ObligationKind::from_tags(obligations)?);
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

        let maker_obligation = if let Some(obligation_kinds) = some_maker_obligation_kinds {
            MakerObligation {
                kinds: obligation_kinds,
                content: maker_order_note.maker_obligation,
            }
        } else {
            let message = format!("Invalid or missing Maker Obligation Kind in Maker Order Note");
            warn!("{}", message);
            return Err(N3xbError::Simple(message));
        };

        let taker_obligation = if let Some(obligation_kinds) = some_taker_obligation_kinds {
            TakerObligation {
                kinds: obligation_kinds,
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
            pubkey: event.pubkey.to_string(),
            event_id: event.id.to_string(),
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
    ) -> Vec<Result<Order<OrderEngineSpecificType>, N3xbError>> {
        let mut orders: Vec<Result<Order<OrderEngineSpecificType>, N3xbError>> = Vec::new();
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
                tag_map.insert(tag.hash_key(), Value::String(trade_uuid_string.to_owned()));
            }
            OrderTag::MakerObligations(obligations) => {
                tag_map.insert(tag.hash_key(), obligations.to_owned().into_iter().collect());
            }
            OrderTag::TakerObligations(obligations) => {
                tag_map.insert(tag.hash_key(), obligations.to_owned().into_iter().collect());
            }
            OrderTag::TradeDetailParameters(parameters) => {
                tag_map.insert(tag.hash_key(), parameters.to_owned().into_iter().collect());
            }
            OrderTag::TradeEngineName(name) => {
                tag_map.insert(
                    tag.hash_key(),
                    Value::Array(vec![Value::String(name.to_owned())]),
                );
            }
            OrderTag::EventKind(kind) => {
                tag_map.insert(
                    tag.hash_key(),
                    Value::Array(vec![Value::String(kind.to_string())]),
                );
            }
            OrderTag::ApplicationTag(app_tag) => {
                tag_map.insert(
                    tag.hash_key(),
                    Value::Array(vec![Value::String(app_tag.to_owned())]),
                );
            }
        });

        Filter::new()
            .kind(Self::MAKER_ORDER_NOTE_KIND)
            .custom(tag_map)
    }

    pub async fn send_taker_offer_message(
        &self,
        pubkey: String,
        maker_order_note_id: String,
        trade_uuid: String,
        offer: Offer<OfferEngineSpecificType>,
    ) -> Result<(), N3xbError> {
        let peer_message = PeerMessage {
            peer_message_id: Option::None,
            maker_order_note_id,
            trade_uuid,
            message_type: PeerMessageType::TakerOffer,
            message: offer,
        };

        let content = PeerMessageContent {
            n3xb_peer_message: peer_message,
        };

        let public_key = XOnlyPublicKey::from_str(&pubkey.as_str()).unwrap();
        let content_string = serde_json::to_string(&content)?;

        let client = self.arc_client.lock().await;
        client.send_direct_msg(public_key, content_string).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    fn send_maker_order_note_expectation(event: Event) -> Result<EventId, Error> {
        print!("Nostr Event: {:?}", event);
        assert!(event.content == SomeTestParams::expected_json_string());
        Ok(event.id)
    }

    #[tokio::test]
    async fn test_send_maker_order_note() {
        let mut client = Client::new();
        client.expect_keys().returning(|| Keys::generate());
        client.expect_subscribe().returning(|_| {});
        client
            .expect_send_event()
            .returning(send_maker_order_note_expectation);

        let arc_client = Arc::new(Mutex::new(client));

        let interface: NostrInterface<
            SomeTradeEngineMakerOrderSpecifics,
            SomeTradeEngineTakerOfferSpecifics,
        > = NostrInterface::new_with_nostr(arc_client, &SomeTestParams::engine_name_str()).await;

        let maker_obligation = MakerObligation {
            kinds: SomeTestParams::maker_obligation_kinds(),
            content: SomeTestParams::maker_obligation_content(),
        };

        let taker_obligation = TakerObligation {
            kinds: SomeTestParams::taker_obligation_kinds(),
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
            pubkey: "".to_string(),
            event_id: "".to_string(),
            trade_uuid: SomeTestParams::some_uuid_string(),
            maker_obligation,
            taker_obligation,
            trade_details,
            trade_engine_specifics,
            pow_difficulty: SomeTestParams::pow_difficulty(),
        };

        interface.send_maker_order_note(order).await.unwrap();
    }

    fn query_order_notes_expectation(
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        let mut tag_set: Vec<OrderTag> = Vec::new();
        tag_set.push(OrderTag::TradeEngineName(SomeTestParams::engine_name_str()));
        tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
        tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));
        let expected_filter = NostrInterface::<
            SomeTradeEngineMakerOrderSpecifics,
            SomeTradeEngineTakerOfferSpecifics,
        >::create_event_tag_filter(tag_set);
        assert!(vec![expected_filter] == filters);

        let expected_timeout = Duration::from_secs(1);
        assert!(expected_timeout == timeout.unwrap());

        let empty_event_vec: Vec<Event> = Vec::new();
        Ok(empty_event_vec)
    }

    #[tokio::test]
    async fn test_query_order_notes() {
        let mut client = Client::new();
        client.expect_keys().returning(|| Keys::generate());
        client.expect_subscribe().returning(|_| {});
        client
            .expect_get_events_of()
            .returning(query_order_notes_expectation);
        let arc_client = Arc::new(Mutex::new(client));

        let interface: NostrInterface<
            SomeTradeEngineMakerOrderSpecifics,
            SomeTradeEngineTakerOfferSpecifics,
        > = NostrInterface::new_with_nostr(arc_client, &SomeTestParams::engine_name_str()).await;

        let _ = interface.query_order_notes().await.unwrap();
    }
}
