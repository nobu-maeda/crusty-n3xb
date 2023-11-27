use log::{debug, error, info, warn};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};
use strum_macros::{Display, IntoStaticStr};
use url::Url;

use tokio::{
    select,
    sync::{mpsc, oneshot},
};

use crate::{
    common::{
        error::{N3xbError, OfferInvalidReason},
        types::{EventIdString, SerdeGenericTrait, SerdeGenericType},
    },
    communicator::CommunicatorAccess,
    offer::{Offer, OfferEnvelope},
    order::Order,
    peer_msg::PeerEnvelope,
    trade_rsp::{TradeResponse, TradeResponseBuilder, TradeResponseStatus},
};

// Maker SM States
pub struct New;
pub struct Pending;
pub struct Trading;

pub struct MakerAccess<State = New> {
    tx: mpsc::Sender<MakerRequest>,
    state: PhantomData<State>,
}

impl MakerAccess<New> {
    pub async fn post_new_order(self) -> Result<MakerAccess<Pending>, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::SendMakerOrder { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()?;

        // Post New Order success, transition to Pending state
        Ok(MakerAccess {
            tx: self.tx,
            state: PhantomData,
        })
    }
}

impl MakerAccess<Pending> {
    pub async fn query_offers(&self) -> HashMap<EventIdString, OfferEnvelope> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<HashMap<EventIdString, OfferEnvelope>>();
        let request = MakerRequest::QueryOffers { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn query_offer(&self, event_id: EventIdString) -> Option<OfferEnvelope> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Option<OfferEnvelope>>();
        let request = MakerRequest::QueryOffer { event_id, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn accept_offer(
        self,
        trade_rsp: TradeResponse,
    ) -> Result<MakerAccess<Trading>, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::AcceptOffer { trade_rsp, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()?;

        // Accept Offer success, transition to Trading state
        Ok(MakerAccess {
            tx: self.tx,
            state: PhantomData,
        })
    }

    pub async fn cancel_order(self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::CancelOrder { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

impl MakerAccess<Trading> {
    pub async fn send_peer_message(
        &self,
        content: Box<dyn SerdeGenericTrait>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::PeerMessage {
            message: content,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn trade_complete(self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::TradeComplete { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

impl<State> MakerAccess<State> {
    pub async fn register_offer_notif_tx(
        &self,
        offer_notif_tx: mpsc::Sender<Result<OfferEnvelope, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::RegisterOfferNotifTx {
            tx: offer_notif_tx,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn unregister_offer_notif_tx(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::UnregisterOfferNotifTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn register_peer_notif_tx(
        &self,
        peer_notif_tx: mpsc::Sender<Result<PeerEnvelope, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::RegisterPeerNotifTx {
            tx: peer_notif_tx,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn unregister_peer_notif_tx(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::UnregisterPeerNotifTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

impl MakerAccess {
    pub(super) async fn new(tx: mpsc::Sender<MakerRequest>) -> Self {
        Self {
            tx,
            state: PhantomData,
        }
    }
}

pub(crate) struct Maker {
    tx: mpsc::Sender<MakerRequest>,
    pub task_handle: tokio::task::JoinHandle<()>,
}

impl Maker {
    const MAKER_REQUEST_CHANNEL_SIZE: usize = 10;

    pub(crate) async fn new(communicator_accessor: CommunicatorAccess, order: Order) -> Self {
        let (tx, rx) = mpsc::channel::<MakerRequest>(Self::MAKER_REQUEST_CHANNEL_SIZE);
        let mut actor = MakerActor::new(rx, communicator_accessor, order).await;
        let task_handle = tokio::spawn(async move { actor.run().await });
        Self { tx, task_handle }
    }

    // Communicator Handle

    pub(crate) async fn new_accessor(&self) -> MakerAccess {
        MakerAccess::new(self.tx.clone()).await
    }
}

#[derive(Display, IntoStaticStr)]
pub(super) enum MakerRequest {
    SendMakerOrder {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    QueryOffers {
        rsp_tx: oneshot::Sender<HashMap<EventIdString, OfferEnvelope>>,
    },
    QueryOffer {
        event_id: EventIdString,
        rsp_tx: oneshot::Sender<Option<OfferEnvelope>>,
    },
    AcceptOffer {
        trade_rsp: TradeResponse,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    CancelOrder {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    PeerMessage {
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    TradeComplete {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterOfferNotifTx {
        tx: mpsc::Sender<Result<OfferEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterOfferNotifTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterPeerNotifTx {
        tx: mpsc::Sender<Result<PeerEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterPeerNotifTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

struct MakerActor {
    rx: mpsc::Receiver<MakerRequest>,
    communicator_accessor: CommunicatorAccess,
    order: Order,
    relay_urls: HashSet<Url>,
    order_event_id: Option<EventIdString>,
    offer_envelopes: HashMap<EventIdString, OfferEnvelope>,
    accepted_offer_event_id: Option<EventIdString>,
    trade_rsp: Option<TradeResponse>,
    trade_rsp_event_id: Option<EventIdString>,
    offer_notif_tx: Option<mpsc::Sender<Result<OfferEnvelope, N3xbError>>>,
    peer_notif_tx: Option<mpsc::Sender<Result<PeerEnvelope, N3xbError>>>,
    reject_invalid_offers_silently: bool,
}

impl MakerActor {
    pub(crate) async fn new(
        rx: mpsc::Receiver<MakerRequest>,
        communicator_accessor: CommunicatorAccess,
        order: Order,
    ) -> Self {
        MakerActor {
            rx,
            communicator_accessor,
            order,
            relay_urls: HashSet::new(),
            order_event_id: None,
            offer_envelopes: HashMap::new(),
            accepted_offer_event_id: None,
            trade_rsp: None,
            trade_rsp_event_id: None,
            offer_notif_tx: None,
            peer_notif_tx: None,
            reject_invalid_offers_silently: true,
        }
    }

    async fn run(&mut self) {
        let (tx, mut rx) = mpsc::channel::<PeerEnvelope>(20);
        let trade_uuid = self.order.trade_uuid;

        if let Some(error) = self
            .communicator_accessor
            .register_peer_message_tx(trade_uuid, tx)
            .await
            .err()
        {
            error!(
                "Failed to register Maker for Peer Messages destined for TradeUUID {}. Maker will terminate. Error: {}",
                trade_uuid, error
            );
        }

        loop {
            select! {
                Some(request) = self.rx.recv() => {
                    if self.handle_request(request).await {
                        break;
                    }
                },
                Some(envelope) = rx.recv() => {
                    self.handle_peer_message(envelope).await;
                },
                else => break,
            }
        }
        info!("Maker w/ TradeUUID {} terminating", trade_uuid);
    }

    // Top-down Request Handling

    async fn handle_request(&mut self, request: MakerRequest) -> bool {
        let mut terminate = false;

        debug!(
            "Maker w/ TradeUUID {} handle_request() of type {}",
            self.order.trade_uuid, request
        );

        match request {
            MakerRequest::SendMakerOrder { rsp_tx } => self.send_maker_order(rsp_tx).await,
            MakerRequest::QueryOffers { rsp_tx } => self.query_offers(rsp_tx).await,
            MakerRequest::QueryOffer { event_id, rsp_tx } => {
                self.query_offer(event_id, rsp_tx).await
            }
            MakerRequest::AcceptOffer { trade_rsp, rsp_tx } => {
                self.accept_offer(trade_rsp, rsp_tx).await
            }
            MakerRequest::CancelOrder { rsp_tx } => {
                self.cancel_order(rsp_tx).await;
                terminate = true;
            }
            MakerRequest::PeerMessage { message, rsp_tx } => {
                self.send_peer_message(message, rsp_tx).await;
            }
            MakerRequest::TradeComplete { rsp_tx } => {
                self.trade_complete(rsp_tx).await;
                terminate = true;
            }
            MakerRequest::RegisterOfferNotifTx { tx, rsp_tx } => {
                self.register_offer_notif_tx(tx, rsp_tx).await;
            }
            MakerRequest::UnregisterOfferNotifTx { rsp_tx } => {
                self.unregister_offer_notif_tx(rsp_tx).await;
            }
            MakerRequest::RegisterPeerNotifTx { tx, rsp_tx } => {
                self.register_peer_notif_tx(tx, rsp_tx).await;
            }
            MakerRequest::UnregisterPeerNotifTx { rsp_tx } => {
                self.unregister_peer_notif_tx(rsp_tx).await;
            }
        };
        terminate
    }

    async fn send_maker_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order = self.order.clone();
        let result = self
            .communicator_accessor
            .send_maker_order_note(order)
            .await;
        match result {
            Ok(order_envelope) => {
                self.order_event_id = Some(order_envelope.event_id);
                self.relay_urls = order_envelope.urls;
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(error) => {
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            }
        }
    }

    async fn query_offers(
        &mut self,
        rsp_tx: oneshot::Sender<HashMap<EventIdString, OfferEnvelope>>,
    ) {
        rsp_tx.send(self.offer_envelopes.clone()).unwrap(); // oneshot should not fail
    }

    async fn query_offer(
        &mut self,
        event_id: EventIdString,
        rsp_tx: oneshot::Sender<Option<OfferEnvelope>>,
    ) {
        let offer = self.offer_envelopes.get(&event_id).cloned();
        rsp_tx.send(offer).unwrap(); // oneshot should not fail
    }

    async fn accept_offer(
        &mut self,
        trade_rsp: TradeResponse,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        if let Some(event_id) = self.accepted_offer_event_id.clone() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} should not have already accepted an Offer. Prev Offer event ID {}, New Offer event ID {}",
                self.order.trade_uuid, event_id, trade_rsp.offer_event_id
            ));
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let accepted_offer_event_id = trade_rsp.offer_event_id.clone();
        self.accepted_offer_event_id = Some(accepted_offer_event_id.clone());

        let pubkey = match self.offer_envelopes.get(&accepted_offer_event_id) {
            Some(offer_envelope) => offer_envelope.pubkey.clone(),
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected, but does not contain accepted Offer {}",
                    self.order.trade_uuid, accepted_offer_event_id
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let maker_order_note_id = match self.order_event_id.clone() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                    self.order.trade_uuid
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        // Send Trade Response Pending to all other Offers
        for offer_envelope in self.offer_envelopes.values() {
            let offer_event_id = offer_envelope.event_id.clone();
            if offer_event_id == accepted_offer_event_id {
                continue;
            } else {
                warn!("Maker w/ TradeUUID {} has other outstanding offers, but no explicit rejection sent to Takers", self.order.trade_uuid)
            }

            // TODO: This triggers Send/Sync problems. Can't find a ready solve. Skipping for now
            // let offer_envelope = offer_envelope.clone();
            //
            // if let Some(reject_err) = self
            //     .reject_taker_offer(offer_envelope, OfferInvalidReason::PendingAnother)
            //     .await
            //     .err()
            // {
            //     error!(
            //         "Maker w/ TradeUUID {} rejected Offer with Event ID {} but with error - {}",
            //         self.order.trade_uuid.clone(),
            //         offer_event_id,
            //         reject_err
            //     )
            // }
        }

        let trade_rsp_clone = trade_rsp.clone();

        let result = self
            .communicator_accessor
            .send_trade_response(
                pubkey,
                Some(accepted_offer_event_id),
                maker_order_note_id,
                self.order.trade_uuid.clone(),
                trade_rsp_clone,
            )
            .await;

        match result {
            Ok(event_id) => {
                self.trade_rsp = Some(trade_rsp);
                self.trade_rsp_event_id = Some(event_id);
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(error) => {
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            }
        }
    }

    async fn cancel_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let maker_order_note_id = match self.order_event_id.clone() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(format!(
                        "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                        self.order.trade_uuid
                    ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        // Send Trade Response Cancelled to all Offers received so far
        for _offer_envelope in self.offer_envelopes.values() {
            warn!("Maker w/ TradeUUID {} has outstanding offers, but no explicit cancellation sent to Takers", self.order.trade_uuid)
            // TODO: This triggers Send/Sync problems. Can't find a ready solve. Skipping for now
            // let pubkey = offer_envelope.pubkey.clone();
            // let offer_event_id = offer_envelope.event_id.clone();

            // let trade_rsp = TradeResponseBuilder::new()
            //     .offer_event_id(offer_event_id.clone())
            //     .trade_response(TradeResponseStatus::Rejected)
            //     .reject_reason(OfferInvalidReason::Cancelled)
            //     .build()
            //     .unwrap();

            // let result = self
            //     .communicator_accessor
            //     .send_trade_response(
            //         pubkey,
            //         Some(offer_event_id.clone()),
            //         maker_order_note_id.clone(),
            //         self.order.trade_uuid.clone(),
            //         trade_rsp,
            //     )
            //     .await;

            // match result {
            //     Ok(_) => {}
            //     Err(error) => {
            //         error!(
            //             "Maker w/ TradeUUID {} failed to send TradeResponse Cancelled to Offer w/ Event ID {} - {}",
            //             self.order.trade_uuid, offer_event_id, error
            //         );
            //     }
            // }
        }

        // Delete Order Note
        let result = self
            .communicator_accessor
            .delete_maker_order_note(maker_order_note_id.clone())
            .await;

        // Send response back to user
        match result {
            Ok(_) => {
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(error) => {
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            }
        }
    }

    async fn send_peer_message(
        &mut self,
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let accepted_offer_event_id = match self.accepted_offer_event_id.clone() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected to already have accepted an Offer",
                    self.order.trade_uuid
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let pubkey = match self.offer_envelopes.get(&accepted_offer_event_id) {
            Some(offer_envelope) => offer_envelope.pubkey.clone(),
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected, but does not contain accepted Offer {}",
                    self.order.trade_uuid, accepted_offer_event_id
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let maker_order_note_id = match self.order_event_id.clone() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                    self.order.trade_uuid
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let result = self
            .communicator_accessor
            .send_trade_engine_specific_message(
                pubkey,
                None,
                maker_order_note_id,
                self.order.trade_uuid.clone(),
                message,
            )
            .await;

        match result {
            Ok(_) => {
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(error) => {
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            }
        }
    }

    async fn trade_complete(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        // TODO: What else to do for Trade Complete?
        rsp_tx.send(Ok(())).unwrap();
    }

    async fn register_offer_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<OfferEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.offer_notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} already have offer_notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.offer_notif_tx = Some(tx);
        rsp_tx.send(result).unwrap();
    }

    async fn unregister_offer_notif_tx(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let mut result = Ok(());
        if self.offer_notif_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} expected to already have offer_notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.offer_notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    async fn register_peer_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<PeerEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.peer_notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} already have peer_notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.peer_notif_tx = Some(tx);
        rsp_tx.send(result).unwrap();
    }

    async fn unregister_peer_notif_tx(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let mut result = Ok(());
        if self.peer_notif_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} expected to already have peer_notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.peer_notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    // Bottom-up Peer Message Handling

    async fn handle_peer_message(&mut self, peer_envelope: PeerEnvelope) {
        debug!(
            "Maker w/ TradeUUID {} handle_peer_message() from pubkey {}, of event id {}, type {:?}",
            self.order.trade_uuid,
            peer_envelope.pubkey.to_string(),
            peer_envelope.event_id.to_string(),
            peer_envelope.message_type
        );

        match peer_envelope.message_type {
            SerdeGenericType::TakerOffer => {
                let offer = peer_envelope.message.downcast_ref::<Offer>().expect(&format!("Maker w/ TradeUUID {} received peer message of SerdeGenericType::TakerOffer, but failed to downcast message into Offer", self.order.trade_uuid)).to_owned();
                let offer_envelope = OfferEnvelope {
                    pubkey: peer_envelope.pubkey,
                    event_id: peer_envelope.event_id,
                    offer,
                    _private: (),
                };
                self.handle_taker_offer(offer_envelope).await;
            }

            SerdeGenericType::TradeResponse => {
                error!(
                    "Maker w/ TradeUUID {} received unexpected TradeResponse message",
                    self.order.trade_uuid
                );
            }

            SerdeGenericType::TradeEngineSpecific => {
                // Let the Trade Engine / user to do the downcasting. Pass the SerdeGeneric message up as is
                if let Some(tx) = &self.peer_notif_tx {
                    if let Some(error) = tx.send(Ok(peer_envelope)).await.err() {
                        error!(
                            "Maker w/ TradeUUID {} failed in notifying user with handle_peer_message - {}",
                            self.order.trade_uuid, error
                        );
                    }
                } else {
                    warn!(
                        "Maker w/ TradeUUID {} do not have Peer notif_tx registered",
                        self.order.trade_uuid
                    );
                }
            }
        }
    }

    async fn handle_taker_offer(&mut self, offer_envelope: OfferEnvelope) {
        let mut notif_result: Result<OfferEnvelope, N3xbError> = Ok(offer_envelope.clone());

        let reason = if self.accepted_offer_event_id.is_some() {
            Some(OfferInvalidReason::PendingAnother)
        } else if self.offer_envelopes.contains_key(&offer_envelope.event_id) {
            Some(OfferInvalidReason::DuplicateOffer)
        } else if let Some(reason) = offer_envelope.offer.validate_against(&self.order).err() {
            Some(reason)
        } else {
            self.offer_envelopes
                .insert(offer_envelope.event_id.clone(), offer_envelope.clone());
            None
        };

        let offer_envelope_clone = offer_envelope.clone();

        if let Some(reason) = reason {
            notif_result = Err(N3xbError::InvalidOffer(reason.clone()));
            if let Some(reject_err) = self
                .reject_taker_offer(offer_envelope_clone, reason)
                .await
                .err()
            {
                error!(
                    "Maker w/ TradeUUID {} rejected Offer with Event ID {} but with error - {}",
                    self.order.trade_uuid, offer_envelope.event_id, reject_err
                )
            }
            if self.reject_invalid_offers_silently {
                return;
            }
        }

        // Notify user of new Offer recieved
        if let Some(tx) = &self.offer_notif_tx {
            if let Some(error) = tx.send(notif_result).await.err() {
                error!(
                    "Maker w/ TradeUUID {} failed in notifying user with handle_taker_offer - {}",
                    self.order.trade_uuid, error
                );
            }
        } else {
            warn!(
                "Maker w/ TradeUUID {} do not have Offer notif_tx registered",
                self.order.trade_uuid
            );
        }
    }

    async fn reject_taker_offer(
        &mut self,
        offer_envelope: OfferEnvelope,
        reason: OfferInvalidReason,
    ) -> Result<(), N3xbError> {
        let mut reject_result: Result<(), N3xbError> = Ok(());

        let pubkey = offer_envelope.pubkey;
        let offer_event_id = offer_envelope.event_id.clone();
        let maker_order_note_id = match self.order_event_id.clone() {
            Some(event_id) => event_id,
            None => {
                reject_result = Err(N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                    self.order.trade_uuid
                )));
                "".to_string()
            }
        };

        let trade_rsp = TradeResponseBuilder::new()
            .offer_event_id(offer_event_id)
            .trade_response(TradeResponseStatus::Rejected)
            .reject_reason(reason.clone())
            .build()
            .unwrap();

        self.communicator_accessor
            .send_trade_response(
                pubkey,
                Some(offer_envelope.event_id.clone()),
                maker_order_note_id,
                self.order.trade_uuid.clone(),
                trade_rsp,
            )
            .await?;

        reject_result
    }
}

#[cfg(test)]
mod tests {
    // TODO: A lot to mock. Postponing this

    // #[tokio::test]
    // async fn test_accept_offer_send_trade_response() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_accept_offer_already_accepted() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_accept_offer_does_not_exist() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_accept_offer_no_order_event_id() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_handle_taker_offer_notify() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_handle_taker_offer_already_accepted() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_handle_taker_offer_invalid_against_order() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_handle_taker_offer_invalid_silently() {
    //     todo!();
    // }
}