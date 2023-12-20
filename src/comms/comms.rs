use log::{debug, error, info, trace, warn};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use secp256k1::{rand::rngs::OsRng, Secp256k1, SecretKey, XOnlyPublicKey};
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::common::error::N3xbError;
use crate::common::types::{EventIdString, ObligationKind, SerdeGenericTrait, SerdeGenericType};
use crate::offer::Offer;
use crate::order::{
    EventKind, FilterTag, MakerObligation, Order, OrderEnvelope, OrderTag, TakerObligation,
    TradeDetails, TradeParameter, N3XB_APPLICATION_TAG,
};
use crate::peer_msg::{PeerEnvelope, PeerMessage};
use crate::trade_rsp::TradeResponse;

use super::data::CommsData;
use super::maker_order_note::MakerOrderNote;
use super::nostr::*;
use super::router::Router;

#[derive(Clone)]
pub(crate) struct CommsAccess {
    tx: mpsc::Sender<CommsRequest>,
}

impl CommsAccess {
    pub(super) fn new(tx: mpsc::Sender<CommsRequest>) -> Self {
        Self { tx }
    }

    pub(crate) async fn get_pubkey(&self) -> XOnlyPublicKey {
        let (rsp_tx, rsp_rx) = oneshot::channel::<XOnlyPublicKey>();
        let request = CommsRequest::GetPublicKey { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn add_relays(
        &self,
        relays: Vec<(url::Url, Option<SocketAddr>)>,
        connect: bool,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::AddRelays {
            relays,
            connect,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn remove_relay(&self, relay: url::Url) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::RemoveRelay { relay, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn get_relays(&self) -> Vec<url::Url> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Vec<url::Url>>();
        let request = CommsRequest::GetRelays { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn connect_relay(&self, relay: url::Url) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::ConnectRelay { relay, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn connect_all_relays(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::ConnectAllRelays { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn register_peer_message_tx(
        &mut self,
        trade_uuid: Uuid,
        tx: mpsc::Sender<PeerEnvelope>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::RegisterTradeTx {
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
        let request = CommsRequest::UnregisterTradeTx { trade_uuid, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn register_peer_message_fallback_tx(
        &mut self,
        tx: mpsc::Sender<PeerEnvelope>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::RegisterFallbackTx { tx, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn unregister_peer_message_fallback_tx(&mut self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::UnregisterFallbackTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn send_maker_order_note(
        &self,
        order: Order,
    ) -> Result<OrderEnvelope, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<OrderEnvelope, N3xbError>>();
        let request = CommsRequest::SendMakerOrderNote { order, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn query_orders(
        &self,
        filter_tags: Vec<FilterTag>,
    ) -> Result<Vec<OrderEnvelope>, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<Vec<OrderEnvelope>, N3xbError>>();
        let request = CommsRequest::QueryOrders {
            filter_tags,
            rsp_tx,
        };
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
        let request = CommsRequest::SendTakerOfferMessage {
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
        let request = CommsRequest::SendTradeResponse {
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

    pub(crate) async fn send_trade_engine_specific_message(
        &self,
        pubkey: XOnlyPublicKey,
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        message: Box<dyn SerdeGenericTrait>,
    ) -> Result<EventIdString, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<EventIdString, N3xbError>>();
        let request = CommsRequest::SendTradeEngineSpecificMessage {
            pubkey,
            responding_to_id,
            maker_order_note_id,
            trade_uuid,
            message,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn delete_maker_order_note(
        &self,
        event_id: EventIdString,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::DeletMakerOrderNote { event_id, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn shutdown(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = CommsRequest::Shutdown { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

pub(crate) struct Comms {
    tx: mpsc::Sender<CommsRequest>,
    pub task_handle: tokio::task::JoinHandle<()>,
}

impl Comms {
    const INTEFACER_REQUEST_CHANNEL_SIZE: usize = 100;
    const NOSTR_EVENT_DEFAULT_POW_DIFFICULTY: u8 = 8;

    // Constructors

    pub(crate) async fn new(
        trade_engine_name: impl Into<String>,
        data_dir_path: impl AsRef<Path>,
    ) -> Self {
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut OsRng);
        Self::new_with_key(secret_key, trade_engine_name, data_dir_path).await
    }

    pub(crate) async fn new_with_key(
        secret_key: SecretKey,
        trade_engine_name: impl Into<String>,
        data_dir_path: impl AsRef<Path>,
    ) -> Self {
        let client = Self::new_nostr_client(secret_key).await;
        Self::new_with_nostr_client(client, trade_engine_name, data_dir_path).await
    }

    pub(super) async fn new_with_nostr_client(
        client: Client,
        trade_engine_name: impl Into<String>,
        data_dir_path: impl AsRef<Path>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<CommsRequest>(Self::INTEFACER_REQUEST_CHANNEL_SIZE);
        let actor = CommsActor::new(rx, trade_engine_name, client, data_dir_path).await;
        let task_handle = tokio::spawn(async move { actor.run().await });
        Self { tx, task_handle }
    }

    async fn new_nostr_client(secret_key: SecretKey) -> Client {
        let keys = Keys::new(secret_key);
        let opts = Options::new()
            .wait_for_connection(true)
            .wait_for_send(true)
            .difficulty(Self::NOSTR_EVENT_DEFAULT_POW_DIFFICULTY);
        Client::with_opts(&keys, opts)
    }

    pub(crate) fn new_accessor(&self) -> CommsAccess {
        CommsAccess::new(self.tx.clone())
    }
}

pub(super) enum CommsRequest {
    // Requests & Arguments
    GetPublicKey {
        rsp_tx: oneshot::Sender<XOnlyPublicKey>,
    },
    AddRelays {
        relays: Vec<(url::Url, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RemoveRelay {
        relay: url::Url,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    GetRelays {
        rsp_tx: oneshot::Sender<Vec<url::Url>>,
    },
    ConnectRelay {
        relay: url::Url,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    ConnectAllRelays {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
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
        rsp_tx: oneshot::Sender<Result<OrderEnvelope, N3xbError>>,
    },
    QueryOrders {
        filter_tags: Vec<FilterTag>,
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
    SendTradeEngineSpecificMessage {
        pubkey: XOnlyPublicKey, // Pubkey of destination receipient
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    },
    DeletMakerOrderNote {
        event_id: EventIdString,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    Shutdown {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

pub(super) struct CommsActor {
    rx: mpsc::Receiver<CommsRequest>,
    trade_engine_name: String,
    pubkey: XOnlyPublicKey,
    data: CommsData,
    client: Client,
    router: Router,
}

impl CommsActor {
    const MAKER_ORDER_NOTE_KIND: Kind = Kind::ParameterizedReplaceable(30078);

    pub(super) async fn new(
        rx: mpsc::Receiver<CommsRequest>,
        trade_engine_name: impl Into<String>,
        client: Client,
        data_dir_path: impl AsRef<Path>,
    ) -> Self {
        let pubkey = client.keys().await.public_key();
        let data = CommsData::new(data_dir_path, pubkey).await.unwrap();
        let relays = data.relays().await;

        let actor = CommsActor {
            rx,
            trade_engine_name: trade_engine_name.into(),
            pubkey,
            data,
            client,
            router: Router::new(),
        };
        actor.add_relays_to_client(relays).await.unwrap();
        actor
    }

    async fn run(mut self) {
        // Nostr client initializaiton
        self.client
            .subscribe(self.subscription_filters(self.pubkey))
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
                        Err(error) => error!("Comms event RX receive error - {}", error),
                    }
                },
                else => break,
            }
        }

        info!("Comms w/ pubkey {} terminating", self.pubkey);
        self.client.shutdown().await.unwrap();
        self.data.terminate().await.unwrap();
    }

    async fn handle_request(&mut self, request: CommsRequest) -> bool {
        let mut terminate = false;

        match request {
            CommsRequest::GetPublicKey { rsp_tx } => self.get_pubkey(rsp_tx).await,

            // Relays Management
            CommsRequest::AddRelays {
                relays,
                connect,
                rsp_tx,
            } => self.add_relays(relays, connect, rsp_tx).await,

            CommsRequest::RemoveRelay { relay, rsp_tx } => self.remove_relay(relay, rsp_tx).await,

            CommsRequest::GetRelays { rsp_tx } => self.get_relays(rsp_tx).await,

            CommsRequest::ConnectRelay { relay, rsp_tx } => self.connect_relay(relay, rsp_tx).await,

            CommsRequest::ConnectAllRelays { rsp_tx } => self.connect_all_relays(rsp_tx).await,

            // Change subscription filters

            // Router management
            CommsRequest::RegisterTradeTx {
                trade_uuid,
                tx,
                rsp_tx,
            } => {
                let result = self.router.register_peer_message_tx(trade_uuid, tx);
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            CommsRequest::UnregisterTradeTx { trade_uuid, rsp_tx } => {
                let result = self.router.unregister_peer_message_tx(trade_uuid);
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            CommsRequest::RegisterFallbackTx { tx, rsp_tx } => {
                let result = self.router.register_peer_message_fallback_tx(tx);
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            CommsRequest::UnregisterFallbackTx { rsp_tx } => {
                let result = self.router.unregister_peer_message_fallback_tx();
                rsp_tx.send(result).unwrap(); // oneshot should never fail
            }

            // Send Maker Order Notes
            CommsRequest::SendMakerOrderNote { order, rsp_tx } => {
                self.send_maker_order_note(order, rsp_tx).await
            }

            // Query Order Notes
            CommsRequest::QueryOrders {
                filter_tags,
                rsp_tx,
            } => self.query_orders(filter_tags, rsp_tx).await,

            // Send Taker Offer Message
            CommsRequest::SendTakerOfferMessage {
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
            CommsRequest::SendTradeResponse {
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

            // Send Trade Engine Specific Peer Message
            CommsRequest::SendTradeEngineSpecificMessage {
                pubkey,
                responding_to_id,
                maker_order_note_id,
                trade_uuid,
                message,
                rsp_tx,
            } => {
                self.send_trade_engine_specific_message(
                    pubkey,
                    responding_to_id,
                    maker_order_note_id,
                    trade_uuid,
                    message,
                    rsp_tx,
                )
                .await;
            }
            // Delete an Maker Order Note
            CommsRequest::DeletMakerOrderNote { event_id, rsp_tx } => {
                self.delete_maker_order_note(event_id, rsp_tx).await;
            }

            // Shutdown
            CommsRequest::Shutdown { rsp_tx } => {
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
                    "Comms w/ pubkey {} handle_notification(), dropping Relay Message from url {}",
                    self.pubkey,
                    url.to_string()
                );
            }
            RelayPoolNotification::Shutdown => {
                info!(
                    "Comms w/ pubkey {} handle_notification() Shutdown",
                    self.pubkey
                );
            }
            RelayPoolNotification::RelayStatus { url, status: _ } => {
                trace!(
                    "Comms w/ pubkey {} handle_notification(), dropping Relay Status from url {}",
                    self.pubkey,
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
                "Comms w/ pubkey {} handle_notification_event() Event kind Fallthrough",
                self.pubkey
            );
        }
    }

    async fn handle_direct_message(&mut self, _url: Url, event: Event) {
        let secret_key = self.client.keys().await.secret_key().unwrap();
        let content = match decrypt(&secret_key, &event.pubkey, &event.content) {
            Ok(content) => content,
            Err(error) => {
                error!(
                    "Comms w/ pubkey {} handle_direct_message() failed to decrypt - {}",
                    self.pubkey, error
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
                        "Comms w/ pubkey {} handle_direct_message() failed in router.handle_peer_message() - {}",
                        self.pubkey,
                        error
                    );
                    return;
                }
            }
            Err(error) => {
                error!(
                    "Comms w/ pubkey {} handle_direct_message() failed to deserialize content as PeerMessage - {}",
                    self.pubkey,
                    error
                );
                return;
            }
        }
    }

    // Nostr Client Management

    async fn get_pubkey(&self, rsp_tx: oneshot::Sender<XOnlyPublicKey>) {
        rsp_tx.send(self.pubkey).unwrap(); // Oneshot should not fail
    }

    async fn add_relays_to_client(
        &self,
        relays: Vec<(url::Url, Option<SocketAddr>)>,
    ) -> Result<(), N3xbError> {
        let into_relays: Vec<(String, Option<SocketAddr>)> = relays
            .clone()
            .into_iter()
            .map(|(url, addr)| {
                let url = url.into();
                (url, addr)
            })
            .collect();
        self.client.add_relays(into_relays).await?;
        Ok(())
    }

    async fn add_relays(
        &self,
        relays: Vec<(url::Url, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        if let Some(error) = self.add_relays_to_client(relays.clone()).await.err() {
            rsp_tx.send(Err(error.into())).unwrap(); // Oneshot should not fail
            return;
        }

        self.data.add_relays(relays).await;

        if connect {
            self.client
                .subscribe(self.subscription_filters(self.pubkey))
                .await;
            self.client.connect().await;
        }
        rsp_tx.send(Ok(())).unwrap(); // Oneshot should not fail
    }

    async fn remove_relay(
        &mut self,
        relay: url::Url,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let relay_string: String = relay.clone().into();
        let result = self.client.remove_relay(relay_string).await;
        match result {
            Ok(_) => {
                rsp_tx.send(Ok(())).unwrap();
                self.data.remove_relay(&relay).await;
            }
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        };
    }

    async fn get_relays(&self, rsp_tx: oneshot::Sender<Vec<url::Url>>) {
        let relays = self.client.relays().await;
        let urls: Vec<url::Url> = relays
            .iter()
            .map(|(url, _)| url::Url::from_str(url.as_str()).unwrap())
            .collect();
        rsp_tx.send(urls).unwrap(); // Oneshot should not fail
    }

    async fn connect_relay(&self, relay: url::Url, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let relay_string = relay.to_string();
        let result = self.client.connect_relay(relay_string).await;
        match result {
            Ok(_) => rsp_tx.send(Ok(())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        };
    }

    async fn connect_all_relays(&self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        self.client.connect().await;
        rsp_tx.send(Ok(())).unwrap();
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
        rsp_tx: oneshot::Sender<Result<OrderEnvelope, N3xbError>>,
    ) {
        // Create Note Content
        let maker_order_note = MakerOrderNote {
            maker_obligation: order.maker_obligation.content.clone(),
            taker_obligation: order.taker_obligation.content.clone(),
            trade_details: order.trade_details.content.clone(),
            trade_engine_specifics: order.trade_engine_specifics.clone(),
            pow_difficulty: order.pow_difficulty.clone(),
        };

        let content_string = match serde_json::to_string(&maker_order_note) {
            Ok(string) => string,
            Err(error) => {
                rsp_tx.send(Err(error.into())).unwrap();
                return;
            }
        };

        let order_tags = OrderTag::from_order(order.clone(), &self.trade_engine_name);

        // NIP-78 Event Kind - 30078
        let builder = EventBuilder::new(
            Self::MAKER_ORDER_NOTE_KIND,
            content_string,
            &Self::create_event_tags(order_tags),
        );

        let keys = self.client.keys().await;

        let urls = self
            .client
            .relays()
            .await
            .keys()
            .cloned()
            .map(|url| url::Url::parse(url.as_str()).unwrap())
            .collect();

        let result = self
            .client
            .send_event(builder.to_event(&keys).unwrap())
            .await;

        match result {
            Ok(event_id) => {
                let order_envelope = OrderEnvelope {
                    pubkey: keys.public_key(),
                    event_id: event_id.to_string(),
                    urls,
                    order,
                    _private: (),
                };
                rsp_tx.send(Ok(order_envelope)).unwrap();
            }
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        }
    }

    fn create_event_tags(tags: Vec<OrderTag>) -> Vec<Tag> {
        tags.iter()
            .map(|event_tag| match event_tag {
                OrderTag::TradeUUID(trade_uuid) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    vec![trade_uuid.to_string()],
                ),
                OrderTag::MakerObligations(obligation_kinds) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    obligation_kinds
                        .to_owned()
                        .iter()
                        .flat_map(|kind| kind.to_tag_strings())
                        .collect(),
                ),
                OrderTag::TakerObligations(obligations_kinds) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    obligations_kinds
                        .to_owned()
                        .iter()
                        .flat_map(|kind| kind.to_tag_strings())
                        .collect(),
                ),
                OrderTag::TradeDetailParameters(parameters) => Tag::Generic(
                    TagKind::Custom(event_tag.key().to_string()),
                    TradeDetails::parameters_to_tags(parameters.clone())
                        .into_iter()
                        .collect(),
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
            .collect()
    }

    // Query Order Notes

    async fn query_orders(
        &self,
        filter_tags: Vec<FilterTag>,
        rsp_tx: oneshot::Sender<Result<Vec<OrderEnvelope>, N3xbError>>,
    ) {
        let order_tags = OrderTag::from_filter_tags(filter_tags, &self.trade_engine_name);

        let filter = Self::create_event_tag_filter(order_tags);
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

            if let Ok(order_tag) = OrderTag::from_key_value(&tag_key, tag_vec) {
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
                    some_maker_obligation_kinds = Some(obligations);
                }
                OrderTag::TakerObligations(obligations) => {
                    some_taker_obligation_kinds = Some(obligations);
                }
                OrderTag::TradeDetailParameters(parameters) => trade_parameters = parameters,

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
            .map(|url| url::Url::parse(url.as_str()).unwrap())
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
                        obligations
                            .to_owned()
                            .into_iter()
                            .flat_map(|kind| kind.to_tag_strings())
                            .collect(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::TakerObligations(obligations) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        obligations
                            .to_owned()
                            .into_iter()
                            .flat_map(|kind| kind.to_tag_strings())
                            .collect(),
                    );
                    Self::consume_tags_for_filter(tags[1..].to_vec(), filter)
                }
                OrderTag::TradeDetailParameters(parameters) => {
                    let filter = filter.custom_tag(
                        Alphabet::try_from(tag.key()).unwrap(),
                        TradeDetails::parameters_to_tags(parameters.clone())
                            .into_iter()
                            .collect(),
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

    async fn send_trade_engine_specific_message(
        &self,
        pubkey: XOnlyPublicKey,
        responding_to_id: Option<EventIdString>,
        maker_order_note_id: EventIdString,
        trade_uuid: Uuid,
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<EventIdString, N3xbError>>,
    ) {
        let peer_message = PeerMessage {
            r#type: "n3xb-peer-message".to_string(),
            responding_to_id,
            maker_order_note_id,
            trade_uuid,
            message_type: SerdeGenericType::TradeEngineSpecific,
            message,
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

    async fn delete_maker_order_note(
        &self,
        event_id: EventIdString,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let result = self
            .client
            .delete_event(
                EventId::from_str(&event_id).unwrap(),
                Some("n3xB: Order cancelled by Maker before Trade commenced"),
            )
            .await;
        match result {
            Ok(_) => rsp_tx.send(Ok(())).unwrap(),
            Err(error) => rsp_tx.send(Err(error.into())).unwrap(),
        }
    }

    async fn shutdown(&self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        info!("Comms w/ pubkey {} Shutdown", self.pubkey);
        // TODO: Any other shutdown logic needed?
        rsp_tx.send(Ok(())).unwrap();
    }
}
