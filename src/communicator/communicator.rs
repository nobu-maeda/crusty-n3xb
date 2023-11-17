use log::{debug, error, info, trace, warn};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc, oneshot};

use secp256k1::{rand::rngs::OsRng, Secp256k1, SecretKey, XOnlyPublicKey};
use url::Url;
use uuid::Uuid;

use crate::common::error::N3xbError;
use crate::common::types::{
    EventIdString, EventKind, ObligationKind, OrderTag, SerdeGenericType, N3XB_APPLICATION_TAG,
};
use crate::offer::Offer;
use crate::order::{
    MakerObligation, Order, OrderEnvelope, TakerObligation, TradeDetails, TradeParameter,
};
use crate::trade_rsp::TradeResponse;

use super::maker_order_note::MakerOrderNote;
use super::nostr::*;
use super::peer_messaging::{PeerEnvelope, PeerMessage};
use super::router::Router;

pub(crate) struct CommunicatorAccess {
    tx: mpsc::Sender<CommunicatorRequest>,
}

impl CommunicatorAccess {
    pub(super) fn new(tx: mpsc::Sender<CommunicatorRequest>) -> Self {
        Self { tx }
    }

    pub(crate) async fn get_pubkey(&self) -> XOnlyPublicKey {
        let (rsp_tx, rsp_rx) = oneshot::channel::<XOnlyPublicKey>();
        let request = CommunicatorRequest::GetPublicKey { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn add_relays(
        &self,
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::AddRelays {
            relays,
            connect,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn remove_relay(&self, relay: impl Into<String>) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::RemoveRelay {
            relay: relay.into(),
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn get_relays(&self) -> Vec<Url> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Vec<Url>>();
        let request = CommunicatorRequest::GetRelays { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn register_peer_message_tx(
        &mut self,
        trade_uuid: Uuid,
        tx: mpsc::Sender<PeerEnvelope>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::RegisterTradeTx {
            trade_uuid,
            tx,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn unregister_peer_message_tx(
        &mut self,
        trade_uuid: Uuid,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::UnregisterTradeTx { trade_uuid, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn register_peer_message_fallback_tx(
        &mut self,
        tx: mpsc::Sender<PeerEnvelope>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::RegisterFallbackTx { tx, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn unregister_peer_message_fallback_tx(&mut self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::UnregisterFallbackTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn send_maker_order_note(
        &self,
        order: Order,
    ) -> Result<EventIdString, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<EventIdString, N3xbError>>();
        let request = CommunicatorRequest::SendMakerOrderNote { order, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn query_orders(&self) -> Result<Vec<OrderEnvelope>, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<Vec<OrderEnvelope>, N3xbError>>();
        let request = CommunicatorRequest::QueryOrders { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn send_taker_offer_message(
        &self,
        pubkey: XOnlyPublicKey, // Pubkey of destination receipient (Maker)
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        offer: Offer,
    ) -> Result<EventIdString, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<EventIdString, N3xbError>>();
        let request = CommunicatorRequest::SendTakerOfferMessage {
            pubkey,
            responding_to_id,
            maker_order_note_id,
            trade_uuid,
            offer,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn send_trade_response(
        &self,
        pubkey: XOnlyPublicKey,
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        trade_rsp: TradeResponse,
    ) -> Result<EventIdString, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<EventIdString, N3xbError>>();
        let request = CommunicatorRequest::SendTradeResponse {
            pubkey,
            responding_to_id,
            maker_order_note_id,
            trade_uuid,
            trade_rsp,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn shutdown(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommunicatorRequest::Shutdown { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

pub(crate) struct Communicator {
    tx: mpsc::Sender<CommunicatorRequest>,
    pub task_handle: tokio::task::JoinHandle<()>,
}

impl Communicator {
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
        let (tx, rx) = mpsc::channel::<CommunicatorRequest>(Self::INTEFACER_REQUEST_CHANNEL_SIZE);
        let actor = CommunicatorActor::new(rx, trade_engine_name, client).await;
        let task_handle = tokio::spawn(async move { actor.run().await });
        Self { tx, task_handle }
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

    // Communicator Handle

    pub(crate) fn new_accessor(&self) -> CommunicatorAccess {
        CommunicatorAccess::new(self.tx.clone())
    }
}

pub(super) enum CommunicatorRequest {
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
    RegisterTradeTx {
        trade_uuid: Uuid,
        tx: mpsc::Sender<PeerEnvelope>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterTradeTx {
        trade_uuid: Uuid,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterFallbackTx {
        tx: mpsc::Sender<PeerEnvelope>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterFallbackTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    SendMakerOrderNote {
        order: Order,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    },
    QueryOrders {
        rsp_tx: oneshot::Sender<Result<Vec<OrderEnvelope>, N3xbError>>,
    },
    SendTakerOfferMessage {
        pubkey: XOnlyPublicKey, // Pubkey of destination receipient (Maker)
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        offer: Offer,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    },
    SendTradeResponse {
        pubkey: XOnlyPublicKey, // Pubkey of destination receipient (Taker)
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        trade_rsp: TradeResponse,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    },
    Shutdown {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

pub(super) struct CommunicatorActor {
    rx: mpsc::Receiver<CommunicatorRequest>,
    trade_engine_name: String,
    client: Client,
    router: Router,
}

impl CommunicatorActor {
    const MAKER_ORDER_NOTE_KIND: Kind = Kind::ParameterizedReplaceable(30078);

    pub(super) async fn new(
        rx: mpsc::Receiver<CommunicatorRequest>,
        trade_engine_name: impl Into<String>,
        client: Client,
    ) -> Self {
        CommunicatorActor {
            rx,
            trade_engine_name: trade_engine_name.into(),
            client,
            router: Router::new(),
        }
    }

    // Event Loop Main

    async fn run(mut self) {
        // Nostr client initializaiton
        let pubkey = self.client.keys().await.public_key();
        self.client
            .subscribe(self.subscription_filters(pubkey))
            .await;

        let mut event_rx = self.client.notifications();

        // Request handling main event loop
        // !!! This function will end if no Sender remains for the Receiver
        loop {
            select! {
                Some(request) = self.rx.recv() => {
                    if self.handle_request(request).await {
                        break;
                    }
                },
                result = event_rx.recv() => {
                    match result {
                        Ok(notification) => self.handle_notification(notification).await,
                        Err(error) => error!("Communicator event RX receive error - {}", error),
                    }
                },
                else => break,
            }
        }

        info!("Communicator w/ pubkey {} terminating", pubkey.to_string());
        self.client.shutdown().await.unwrap();
    }

    async fn handle_request(&mut self, request: CommunicatorRequest) -> bool {
        let mut terminate = false;

        match request {
            CommunicatorRequest::GetPublicKey { rsp_tx } => self.get_pubkey(rsp_tx).await,

            // Relays Management
            CommunicatorRequest::AddRelays {
                relays,
                connect,
                rsp_tx,
            } => self.add_relays(relays, connect, rsp_tx).await,

            CommunicatorRequest::RemoveRelay { relay, rsp_tx } => {
                self.remove_relay(relay, rsp_tx).await
            }

            CommunicatorRequest::GetRelays { rsp_tx } => self.get_relays(rsp_tx).await,

            // Change subscription filters

            // Router management
            CommunicatorRequest::RegisterTradeTx {
                trade_uuid,
                tx,
                rsp_tx,
            } => {
                let result = self.router.register_peer_message_tx(trade_uuid, tx);
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            CommunicatorRequest::UnregisterTradeTx { trade_uuid, rsp_tx } => {
                let result = self.router.unregister_peer_message_tx(trade_uuid);
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            CommunicatorRequest::RegisterFallbackTx { tx, rsp_tx } => {
                let result = self.router.register_peer_message_fallback_tx(tx);
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            CommunicatorRequest::UnregisterFallbackTx { rsp_tx } => {
                let result = self.router.unregister_peer_message_fallback_tx();
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            // Send Maker Order Notes
            CommunicatorRequest::SendMakerOrderNote { order, rsp_tx } => {
                self.send_maker_order_note(order, rsp_tx).await
            }

            // Query Order Notes
            CommunicatorRequest::QueryOrders { rsp_tx } => self.query_orders(rsp_tx).await,

            // Send Taker Offer Message
            CommunicatorRequest::SendTakerOfferMessage {
                pubkey,
                responding_to_id,
                maker_order_note_id,
                trade_uuid,
                offer,
                rsp_tx,
            } => {
                self.send_taker_offer_message(
                    pubkey,
                    responding_to_id,
                    maker_order_note_id,
                    trade_uuid,
                    offer,
                    rsp_tx,
                )
                .await;
            }

            // Send Trade Response
            CommunicatorRequest::SendTradeResponse {
                pubkey,
                responding_to_id,
                maker_order_note_id,
                trade_uuid,
                trade_rsp,
                rsp_tx,
            } => {
                self.send_trade_response(
                    pubkey,
                    responding_to_id,
                    maker_order_note_id,
                    trade_uuid,
                    trade_rsp,
                    rsp_tx,
                )
                .await;
            }

            // Shutdown
            CommunicatorRequest::Shutdown { rsp_tx } => {
                self.shutdown(rsp_tx).await;
                terminate = true;
            }
        }
        terminate
    }

    async fn handle_notification(&mut self, notification: RelayPoolNotification) {
        match notification {
            RelayPoolNotification::Event(url, event) => {
                self.handle_notification_event(Url::from_str(url.as_str()).unwrap(), event)
                    .await;
            }
            RelayPoolNotification::Message(url, _relay_message) => {
                trace!(
                    "Communicator w/ pubkey {} handle_notification(), dropping Relay Message from url {}",
                    self.client.keys().await.public_key().to_string(),
                    url.to_string()
                );
            }
            RelayPoolNotification::Shutdown => {
                info!(
                    "Communicator w/ pubkey {} handle_notification() Shutdown",
                    self.client.keys().await.public_key().to_string()
                );
            }
            RelayPoolNotification::RelayStatus { url, status: _ } => {
                trace!(
                    "Communicator w/ pubkey {} handle_notification(), dropping Relay Status from url {}",
                    self.client.keys().await.public_key().to_string(),
                    url.to_string()
                );
            }
            RelayPoolNotification::Stop => todo!(),
        };
    }

    async fn handle_notification_event(&mut self, url: Url, event: Event) {
        if let Kind::EncryptedDirectMessage = event.kind {
            self.handle_direct_message(url, event).await;
        } else {
            debug!(
                "Communicator w/ pubkey {} handle_notification_event() Event kind Fallthrough",
                self.client.keys().await.public_key().to_string()
            );
        }
    }

    async fn handle_direct_message(&mut self, _url: Url, event: Event) {
        let secret_key = self.client.keys().await.secret_key().unwrap();
        let content = match decrypt(&secret_key, &event.pubkey, &event.content) {
            Ok(content) => content,
            Err(error) => {
                error!(
                    "Communicator w/ pubkey {} handle_direct_message() failed to decrypt - {}",
                    self.client.keys().await.public_key().to_string(),
                    error
                );
                return;
            }
        };

        match serde_json::from_str::<PeerMessage>(content.as_str()) {
            Ok(peer_message) => {
                if let Some(error) = self
                    .router
                    .handle_peer_message(event.pubkey, event.id.to_string(), peer_message)
                    .await
                    .err()
                {
                    error!(
                        "Communicator w/ pubkey {} handle_direct_message() failed in router.handle_peer_message() - {}",
                        self.client.keys().await.public_key().to_string(),
                        error
                    );
                    return;
                }
            }
            Err(error) => {
                error!(
                    "Communicator w/ pubkey {} handle_direct_message() failed to deserialize content as PeerMessage - {}",
                    self.client.keys().await.public_key().to_string(),
                    error
                );
                return;
            }
        }
    }

    // Nostr Client Management

    async fn get_pubkey(&self, rsp_tx: oneshot::Sender<XOnlyPublicKey>) {
        let pubkey = self.client.keys().await.public_key();
        rsp_tx.send(pubkey).unwrap(); // Oneshot should not fail
    }

    async fn add_relays(
        &mut self,
        relays: Vec<(impl Into<String> + 'static, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let into_relays: Vec<(String, Option<SocketAddr>)> = relays
            .into_iter()
            .map(|(url, addr)| {
                let url = url.into();
                (url, addr)
            })
            .collect();
        if let Some(error) = self.client.add_relays(into_relays).await.err() {
            rsp_tx.send(Err(error.into())).unwrap(); // Oneshot should not fail
            return;
        }

        if connect {
            let pubkey = self.client.keys().await.public_key();
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
        let relay_string: String = relay.into();
        let result = self.client.remove_relay(relay_string).await;
        match result {
            Ok(_) => rsp_tx.send(Ok(())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        };
    }

    async fn get_relays(&self, rsp_tx: oneshot::Sender<Vec<Url>>) {
        let relays = self.client.relays().await;
        let urls: Vec<Url> = relays
            .iter()
            .map(|(url, _)| Url::from_str(url.as_str()).unwrap())
            .collect();
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
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    ) {
        // Create Note Content
        let maker_order_note = MakerOrderNote {
            maker_obligation: order.maker_obligation.content,
            taker_obligation: order.taker_obligation.content,
            trade_details: order.trade_details.content,
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

        tag_set.push(OrderTag::TradeUUID(order.trade_uuid));
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
            TradeDetails::parameters_to_tags(order.trade_details.parameters),
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

        let keys = self.client.keys().await;
        let result = self
            .client
            .send_event(builder.to_event(&keys).unwrap())
            .await;

        match result {
            Ok(event_id) => rsp_tx.send(Ok(event_id.to_string())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        }
    }

    fn create_event_tags(tags: Vec<OrderTag>) -> Vec<Tag> {
        let tags_vec = tags
            .iter()
            .map(|event_tag| match event_tag {
                OrderTag::TradeUUID(trade_uuid) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    vec![trade_uuid.to_string()],
                ),
                OrderTag::MakerObligations(obligations) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    obligations.to_owned().into_iter().collect(),
                ),
                OrderTag::TakerObligations(obligations) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    obligations.to_owned().into_iter().collect(),
                ),
                OrderTag::TradeDetailParameters(parameters) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    parameters.to_owned().into_iter().collect(),
                ),
                OrderTag::TradeEngineName(name) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    vec![name.to_owned()],
                ),
                OrderTag::EventKind(kind) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    vec![kind.to_string()],
                ),
                OrderTag::ApplicationTag(app_tag) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    vec![app_tag.to_owned()],
                ),
            })
            .collect();
        print!("{:?} ", tags_vec);
        tags_vec
    }

    // Query Order Notes

    async fn query_orders(&self, rsp_tx: oneshot::Sender<Result<Vec<OrderEnvelope>, N3xbError>>) {
        let mut tag_set: Vec<OrderTag> = Vec::new();
        tag_set.push(OrderTag::TradeEngineName(self.trade_engine_name.to_owned()));
        tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
        tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));

        // TODO: Add ways to filter for
        //  - Trade Engine Name
        //  - Maker Obligation Kind
        //  - Taker Obligation Kind
        //  - Trade Detail Parameters

        let filter = Self::create_event_tag_filter(tag_set);
        let timeout = Duration::from_secs(1);
        let events = match self.client.get_events_of(vec![filter], Some(timeout)).await {
            Ok(events) => events,
            Err(error) => {
                rsp_tx.send(Err(error.into())).unwrap();
                return;
            }
        };

        let maybe_order_envelopes = self.extract_order_envelopes_from_events(events).await;
        let mut order_envelopes: Vec<OrderEnvelope> = Vec::new();
        for maybe_order_envelope in maybe_order_envelopes {
            match maybe_order_envelope {
                Ok(order_envelope) => order_envelopes.push(order_envelope),
                Err(error) => {
                    warn!(
                        "Order extraction from Nostr event failed - {}",
                        error.to_string()
                    );
                }
            }
        }
        rsp_tx.send(Ok(order_envelopes)).unwrap();
    }

    fn extract_order_tags_from_tags(&self, tags: Vec<Tag>) -> Vec<OrderTag> {
        let mut order_tags: Vec<OrderTag> = Vec::new();
        for tag in tags {
            let mut tag_vec = tag.as_vec();
            let tag_key = tag_vec.remove(0);

            if let Ok(order_tag) = OrderTag::from_key(&tag_key, tag_vec) {
                order_tags.push(order_tag);
            } else {
                warn!("Unrecognized Tag with key: {}", tag_key);
            }
        }
        order_tags
    }

    async fn extract_order_envelope_from_event(
        &self,
        event: Event,
    ) -> Result<OrderEnvelope, N3xbError> {
        let maker_order_note: MakerOrderNote = serde_json::from_str(event.content.as_str())?;
        let order_tags = self.extract_order_tags_from_tags(event.tags);

        let mut some_trade_uuid: Option<Uuid> = None;
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

        let order = Order {
            trade_uuid,
            maker_obligation,
            taker_obligation,
            trade_details,
            trade_engine_specifics: maker_order_note.trade_engine_specifics,
            pow_difficulty: maker_order_note.pow_difficulty,
            _private: (),
        };

        // Is this order seen from other relays?
        let relay_urls = self
            .client
            .database()
            .event_recently_seen_on_relays(event.id)
            .await
            .unwrap()
            .unwrap();

        let urls = relay_urls
            .iter()
            .map(|url| Url::parse(url.as_str()).unwrap())
            .collect();

        Ok(OrderEnvelope {
            pubkey: event.pubkey,
            urls,
            event_id: event.id.to_string(),
            order: order,
            _private: (),
        })
    }

    async fn extract_order_envelopes_from_events(
        &self,
        events: Vec<Event>,
    ) -> Vec<Result<OrderEnvelope, N3xbError>> {
        let mut order_envelopes: Vec<Result<OrderEnvelope, N3xbError>> = Vec::new();
        let mut event_ids: HashSet<EventId> = HashSet::new();

        for event in events {
            // See if this event have been seen from another relay
            // Bypass if so because it should have been accounted for the first time
            if event_ids.contains(&event.id) {
                continue;
            } else {
                event_ids.insert(event.id);
            }

            let order_envelope = self.extract_order_envelope_from_event(event).await;
            order_envelopes.push(order_envelope);
        }
        order_envelopes
    }

    fn consume_tags_for_filter(tags: Vec<OrderTag>, filter: Filter) -> Filter {
        if let Some(tag) = tags.first() {
            match tag {
                OrderTag::TradeUUID(trade_uuid) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        [trade_uuid.to_string()].to_vec(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::MakerObligations(obligations) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        obligations.to_owned().into_iter().collect(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::TakerObligations(obligations) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        obligations.to_owned().into_iter().collect(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::TradeDetailParameters(parameters) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        parameters.to_owned().into_iter().collect(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::TradeEngineName(name) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        [name.to_owned()].to_vec(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::EventKind(kind) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        [kind.to_string()].to_vec(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::ApplicationTag(app_tag) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        [app_tag.to_owned()].to_vec(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
            }
        } else {
            filter
        }
    }

    fn create_event_tag_filter(tags: Vec<OrderTag>) -> Filter {
        let filter = Filter::new().kind(Self::MAKER_ORDER_NOTE_KIND);
        Self::consume_tags_for_filter(tags, filter)
    }

    async fn send_peer_message(
        &self,
        pubkey: XOnlyPublicKey,
        peer_message: PeerMessage,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    ) {
        let content_string = match serde_json::to_string(&peer_message) {
            Ok(string) => string,
            Err(error) => {
                rsp_tx.send(Err(error.into())).unwrap();
                return;
            }
        };

        let responding_to_event_id: Option<EventId> =
            if let Some(responding_to_id) = peer_message.responding_to_id {
                Some(EventId::from_str(responding_to_id.as_str()).unwrap())
            } else {
                None
            };

        let result = self
            .client
            .send_direct_msg(pubkey, content_string, responding_to_event_id)
            .await;

        match result {
            Ok(event_id) => rsp_tx.send(Ok(event_id.to_string())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        }
    }

    async fn send_taker_offer_message(
        &self,
        pubkey: XOnlyPublicKey,
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        offer: Offer,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    ) {
        let peer_message = PeerMessage {
            r#type: "n3xb-peer-message".to_string(),
            responding_to_id,
            maker_order_note_id,
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        self.send_peer_message(pubkey, peer_message, rsp_tx).await;
    }

    async fn send_trade_response(
        &self,
        pubkey: XOnlyPublicKey,
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        trade_rsp: TradeResponse,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    ) {
        let peer_message = PeerMessage {
            r#type: "n3xb-peer-message".to_string(),
            responding_to_id,
            maker_order_note_id,
            trade_uuid,
            message_type: SerdeGenericType::TradeResponse,
            message: Box::new(trade_rsp),
        };

        self.send_peer_message(pubkey, peer_message, rsp_tx).await;
    }

    async fn shutdown(&self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        info!(
            "Communicator w/ pubkey {} Shutdown",
            self.client.keys().await.public_key().to_string()
        );
        // TODO: Any other shutdown logic needed?
        rsp_tx.send(Ok(())).unwrap();
    }
}
