use log::debug;
use std::net::SocketAddr;

use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey};
use tokio::sync::{mpsc, oneshot};

use crate::common::error::N3xbError;
use crate::common::types::{EventKind, OrderTag, N3XB_APPLICATION_TAG};
use crate::order::Order;

use super::maker_order_note::MakerOrderNote;
use super::nostr::*;

pub(crate) struct InterfacerHandle {
    tx: mpsc::Sender<InterfacerRequest>,
}

impl InterfacerHandle {
    pub(super) fn new(tx: mpsc::Sender<InterfacerRequest>) -> Self {
        InterfacerHandle { tx }
    }

    pub(crate) async fn get_public_key(&self) -> XOnlyPublicKey {
        let (rsp_tx, rsp_rx) = oneshot::channel::<XOnlyPublicKey>();
        let request = InterfacerRequest::GetPublicKey { rsp_tx };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn add_relays(
        &self,
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = InterfacerRequest::AddRelays {
            relays,
            connect,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()?;
        Ok(())
    }

    pub(crate) async fn remove_relay(&self, relay: impl Into<String>) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = InterfacerRequest::RemoveRelay {
            relay: relay.into(),
            rsp_tx,
        };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()?;
        Ok(())
    }

    pub(crate) async fn get_relays(&self) -> Vec<Url> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Vec<Url>>();
        let request = InterfacerRequest::GetRelays { rsp_tx };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn send_maker_order_note(&self, order: Order) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = InterfacerRequest::SendMakerOrderNote { order, rsp_tx };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()
    }
}

pub(crate) struct Interfacer {
    tx: mpsc::Sender<InterfacerRequest>,
}

impl Interfacer {
    const INTEFACER_REQUEST_CHANNEL_SIZE: usize = 100;
    const NOSTR_EVENT_DEFAULT_POW_DIFFICULTY: u8 = 8;

    // Constructors

    pub(crate) async fn new(trade_engine_name: impl Into<String>) -> Self {
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut OsRng);
        Self::new_with_key(secret_key, trade_engine_name).await
    }

    pub(crate) async fn new_with_key(
        secret_key: SecretKey,
        trade_engine_name: impl Into<String>,
    ) -> Self {
        let client = Self::new_nostr_client(secret_key).await;
        Self::new_with_nostr_client(client, trade_engine_name).await
    }

    pub(super) async fn new_with_nostr_client(
        client: Client,
        trade_engine_name: impl Into<String>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<InterfacerRequest>(Self::INTEFACER_REQUEST_CHANNEL_SIZE);
        let mut actor = InterfacerActor::new(rx, trade_engine_name, client).await;
        tokio::spawn(async move { actor.run().await });
        Self { tx }
    }

    async fn new_nostr_client(secret_key: SecretKey) -> Client {
        let keys = Keys::new(secret_key);
        let opts = Options::new()
            .wait_for_connection(true)
            .wait_for_send(true)
            .difficulty(Self::NOSTR_EVENT_DEFAULT_POW_DIFFICULTY);
        let client = Client::with_opts(&keys, opts);
        // TODO: Add saved or default clients
        client.connect().await;
        client
    }

    // Interfacer Handle

    pub(crate) fn get_handle(&self) -> InterfacerHandle {
        InterfacerHandle::new(self.tx.clone())
    }
}

pub(super) enum InterfacerRequest {
    // Requests & Arguments
    GetPublicKey {
        rsp_tx: oneshot::Sender<XOnlyPublicKey>,
    },
    AddRelays {
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RemoveRelay {
        relay: String,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    GetRelays {
        rsp_tx: oneshot::Sender<Vec<Url>>,
    },
    SendMakerOrderNote {
        order: Order,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}
pub(super) struct InterfacerActor {
    rx: mpsc::Receiver<InterfacerRequest>,
    trade_engine_name: String,
    client: Client,
}

impl InterfacerActor {
    const MAKER_ORDER_NOTE_KIND: Kind = Kind::ParameterizedReplaceable(30078);

    pub(super) async fn new(
        rx: mpsc::Receiver<InterfacerRequest>,
        trade_engine_name: impl Into<String>,
        client: Client,
    ) -> Self {
        InterfacerActor {
            rx,
            trade_engine_name: trade_engine_name.into(),
            client,
        }
    }

    // Event Loop Main

    async fn run(&mut self) {
        // Nostr client initializaiton
        let pubkey = self.client.keys().public_key();
        self.client
            .subscribe(self.subscription_filters(pubkey))
            .await;

        // Request handling main event loop
        // !!! This function will end if no Sender remains for the Receiver
        while let Some(request) = self.rx.recv().await {
            self.handle_request(request).await;
        }
        debug!("Interfacer reqeust handling main loop ended");
    }

    async fn handle_request(&mut self, request: InterfacerRequest) {
        match request {
            InterfacerRequest::GetPublicKey { rsp_tx } => self.get_public_key(rsp_tx),

            // Change Relays
            InterfacerRequest::AddRelays {
                relays,
                connect,
                rsp_tx,
            } => self.add_relays(relays, connect, rsp_tx).await,

            InterfacerRequest::RemoveRelay { relay, rsp_tx } => {
                self.remove_relay(relay, rsp_tx).await
            }

            InterfacerRequest::GetRelays { rsp_tx } => self.get_relays(rsp_tx).await,

            // Change subscription filters

            // Send Maker Order Notes
            InterfacerRequest::SendMakerOrderNote { order, rsp_tx } => {
                self.send_maker_order_note(order, rsp_tx).await
            } // Query Order Notes

              // Send Taker Offer Message
        }
    }

    // Nostr Client Management

    fn get_public_key(&self, rsp_tx: oneshot::Sender<XOnlyPublicKey>) {
        let public_key = self.client.keys().public_key();
        rsp_tx.send(public_key).unwrap(); // Oneshot should not fail
    }

    async fn add_relays(
        &mut self,
        relays: Vec<(impl Into<String> + 'static, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        if let Some(error) = self.client.add_relays(relays).await.err() {
            rsp_tx.send(Err(error.into())).unwrap(); // Oneshot should not fail
            return;
        }

        if connect {
            let pubkey = self.client.keys().public_key();
            self.client
                .subscribe(self.subscription_filters(pubkey))
                .await;
            self.client.connect().await;
        }
        rsp_tx.send(Ok(())).unwrap(); // Oneshot should not fail
    }

    async fn remove_relay(
        &mut self,
        relay: impl Into<String> + 'static,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let result = self.client.remove_relay(relay).await;
        match result {
            Ok(_) => rsp_tx.send(Ok(())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        };
    }

    async fn get_relays(&self, rsp_tx: oneshot::Sender<Vec<Url>>) {
        let relays = self.client.relays().await;
        let urls: Vec<Url> = relays.iter().map(|(url, _)| url.to_owned()).collect();
        rsp_tx.send(urls).unwrap(); // Oneshot should not fail
    }

    fn subscription_filters(&self, pubkey: XOnlyPublicKey) -> Vec<Filter> {
        // Need a way to track existing Filters
        // Need a way to correlate State Machines to Subscriptions as to remove filters as necessary

        // Subscribe to all DM to own pubkey. Filter unrecognized DM out some other way. Can be spam prone
        let dm_filter = Filter::new().since(Timestamp::now()).pubkey(pubkey);
        vec![dm_filter]
    }

    // Send Maker Order Note

    async fn send_maker_order_note(
        &self,
        order: Order,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        // Create Note Content
        let maker_order_note = MakerOrderNote {
            maker_obligation: order.maker_obligation.content.to_owned(),
            taker_obligation: order.taker_obligation.content.to_owned(),
            trade_details: order.trade_details.content.to_owned(),
            trade_engine_specifics: order.trade_engine_specifics,
            pow_difficulty: order.pow_difficulty,
        };

        let content_string = match serde_json::to_string(&maker_order_note) {
            Ok(string) => string,
            Err(error) => {
                rsp_tx.send(Err(error.into())).unwrap();
                return;
            }
        };

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

        let keys = self.client.keys();
        let result = self
            .client
            .send_event(builder.to_event(&keys).unwrap())
            .await;

        match result {
            Ok(_) => rsp_tx.send(Ok(())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        }
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        order::{MakerObligation, TakerObligation, TradeDetails},
        testing::*,
    };

    #[tokio::test]
    async fn test_get_public_key() {
        let mut client = Client::new();
        client.expect_keys().returning(|| {
            let secret_key = SomeTestParams::some_secret_key();
            Keys::new(secret_key)
        });
        client.expect_subscribe().returning(|_| {});

        let interfacer =
            Interfacer::new_with_nostr_client(client, SomeTestParams::engine_name_str()).await;
        let interfacer_handle = interfacer.get_handle();

        let secret_key = SomeTestParams::some_secret_key();
        let keys = Keys::new(secret_key);
        let public_key = keys.public_key();
        assert_eq!(public_key, interfacer_handle.get_public_key().await);
    }

    #[tokio::test]
    async fn test_send_maker_order_note() {
        let mut client = Client::new();
        client.expect_keys().returning(|| Keys::generate());
        client.expect_subscribe().returning(|_| {});
        client
            .expect_send_event()
            .returning(send_maker_order_note_expectation);

        let interfacer =
            Interfacer::new_with_nostr_client(client, SomeTestParams::engine_name_str()).await;
        let interfacer_handle = interfacer.get_handle();

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

        let rc_trade_engine_specifics = Arc::new(trade_engine_specifics);

        let order = Order {
            pubkey: "".to_string(),
            event_id: "".to_string(),
            trade_uuid: SomeTestParams::some_uuid_string(),
            maker_obligation,
            taker_obligation,
            trade_details,
            trade_engine_specifics: rc_trade_engine_specifics,
            pow_difficulty: SomeTestParams::pow_difficulty(),
        };

        interfacer_handle
            .send_maker_order_note(order)
            .await
            .unwrap();
    }

    fn send_maker_order_note_expectation(event: Event) -> Result<EventId, Error> {
        print!("Nostr Event: {:?}\n", event);
        print!("Nostr Event Content: {:?}\n", event.content);
        assert!(event.content == SomeTestParams::expected_json_string());
        Ok(event.id)
    }
}
