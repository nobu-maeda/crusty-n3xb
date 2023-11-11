use log::error;
use std::collections::HashMap;

use tokio::{
    select,
    sync::{mpsc, oneshot},
};

use crate::{
    common::{
        error::N3xbError,
        types::{EventIdString, SerdeGenericType},
    },
    interfacer::{InterfacerHandle, PeerEnvelope},
    offer::{Offer, OfferEnvelope},
    order::Order,
    trade_rsp::TradeResponse,
};

pub struct Maker {
    tx: mpsc::Sender<MakerRequest>,
}

impl Maker {
    pub(super) async fn new(tx: mpsc::Sender<MakerRequest>) -> Self {
        let maker = Self { tx };
        maker.make_new_order().await.unwrap();
        maker
    }

    pub async fn make_new_order(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::SendMakerOrder { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

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

    pub async fn accept_offer(&self, trade_rsp: TradeResponse) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::AcceptOffer { trade_rsp, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn register_offer_notif_tx(
        &self,
        notif_tx: mpsc::Sender<Result<OfferEnvelope, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::RegisterOfferNotifTx {
            tx: notif_tx,
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
}

pub(crate) struct MakerEngine {
    tx: mpsc::Sender<MakerRequest>,
}

impl MakerEngine {
    const MAKER_REQUEST_CHANNEL_SIZE: usize = 2;

    pub(crate) async fn new(interfacer_handle: InterfacerHandle, order: Order) -> Self {
        let (tx, rx) = mpsc::channel::<MakerRequest>(Self::MAKER_REQUEST_CHANNEL_SIZE);
        let mut actor = MakerActor::new(rx, interfacer_handle, order).await;
        tokio::spawn(async move { actor.run().await });
        Self { tx }
    }

    // Interfacer Handle

    pub(crate) async fn new_handle(&self) -> Maker {
        Maker::new(self.tx.clone()).await
    }
}

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
    RegisterOfferNotifTx {
        tx: mpsc::Sender<Result<OfferEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterOfferNotifTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

struct MakerActor {
    rx: mpsc::Receiver<MakerRequest>,
    interfacer_handle: InterfacerHandle,
    order: Order,
    order_event_id: Option<EventIdString>,
    offer_envelopes: HashMap<EventIdString, OfferEnvelope>,
    accepted_offer_event_id: Option<EventIdString>,
    trade_rsp: Option<TradeResponse>,
    trade_rsp_event_id: Option<EventIdString>,
    notif_tx: Option<mpsc::Sender<Result<OfferEnvelope, N3xbError>>>,
}

impl MakerActor {
    pub(crate) async fn new(
        rx: mpsc::Receiver<MakerRequest>,
        interfacer_handle: InterfacerHandle,
        order: Order,
    ) -> Self {
        MakerActor {
            rx,
            interfacer_handle,
            order,
            order_event_id: None,
            offer_envelopes: HashMap::new(),
            accepted_offer_event_id: None,
            trade_rsp: None,
            trade_rsp_event_id: None,
            notif_tx: None,
        }
    }

    async fn run(&mut self) {
        let (tx, mut rx) = mpsc::channel::<PeerEnvelope>(20);
        let trade_uuid = self.order.trade_uuid;

        if let Some(error) = self
            .interfacer_handle
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
                    self.handle_request(request).await;
                },
                Some(envelope) = rx.recv() => {
                    self.handle_peer_message(envelope).await;
                },
                else => break,
            }
        }
    }

    async fn handle_request(&mut self, request: MakerRequest) {
        match request {
            MakerRequest::SendMakerOrder { rsp_tx } => self.send_maker_order(rsp_tx).await,
            MakerRequest::QueryOffers { rsp_tx } => self.query_offers(rsp_tx).await,
            MakerRequest::QueryOffer { event_id, rsp_tx } => {
                self.query_offer(event_id, rsp_tx).await
            }
            MakerRequest::AcceptOffer { trade_rsp, rsp_tx } => {
                self.accept_offer(trade_rsp, rsp_tx).await
            }
            MakerRequest::RegisterOfferNotifTx { tx, rsp_tx } => {
                self.register_notif_tx(tx, rsp_tx).await;
            }
            MakerRequest::UnregisterOfferNotifTx { rsp_tx } => {
                self.unregister_notif_tx(rsp_tx).await;
            }
        }
    }

    async fn send_maker_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order = self.order.clone();
        let result = self.interfacer_handle.send_maker_order_note(order).await;
        match result {
            Ok(event_id) => {
                self.order_event_id = Some(event_id.clone());
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
                "Maker with TradeUUID {} should not have already accepted an Offer. Prev Offer event ID {}, New Offer event ID {}",
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
                    "Maker with TradeUUID {} expected to contain accepted Offer {}",
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
                    "Maker with TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                    self.order.trade_uuid
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let trade_rsp_clone = trade_rsp.clone();

        let result = self
            .interfacer_handle
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

    async fn register_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<OfferEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Maker with TradeUUID {} already have notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = Some(tx);
        rsp_tx.send(result).unwrap();
    }

    async fn unregister_notif_tx(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let mut result = Ok(());
        if self.notif_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "Maker with TradeUUID {} expected to already have notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    async fn handle_peer_message(&mut self, peer_envelope: PeerEnvelope) {
        match peer_envelope.message_type {
            SerdeGenericType::TakerOffer => {
                let offer = peer_envelope.message.downcast_ref::<Offer>().expect(&format!("Maker with TradeUUID {} received peer message of SerdeGenericType::TakerOffer, but failed to downcast message into Offer", self.order.trade_uuid)).to_owned();
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
                    "Maker with TradeUUID {} received unexpected TradeResponse message",
                    self.order.trade_uuid
                );
            }

            SerdeGenericType::TradeEngineSpecific => {
                todo!();
            }
        }
    }

    async fn handle_taker_offer(&mut self, offer_envelope: OfferEnvelope) {
        let mut notif_result: Result<OfferEnvelope, N3xbError> = Ok(offer_envelope.clone());

        let valid = offer_envelope.offer.validate_against(&self.order);
        match valid {
            Ok(_) => {
                if let Some(accepted_offer_event_id) = self.accepted_offer_event_id.clone() {
                    notif_result = Err(N3xbError::Simple(format!(
                        "Maker with TradeUUID {} received Offer with EventID {} after already accepted Offer with EventID {}",
                        self.order.trade_uuid, offer_envelope.event_id, accepted_offer_event_id
                    )));
                } else if self.offer_envelopes.contains_key(&offer_envelope.event_id) {
                    notif_result = Err(N3xbError::Simple(format!(
                        "Maker with TradeUUID {} received duplicate Offer with EventID {}",
                        self.order.trade_uuid, offer_envelope.event_id
                    )));
                } else {
                    self.offer_envelopes
                        .insert(offer_envelope.event_id.clone(), offer_envelope);
                }
            }

            Err(_error) => {
                // TODO: Reject offer by sending Taker a Trade Response message
                // Optionally notify user of invalid Offer received
            }
        }

        // Notify user of new Offer recieved
        if let Some(tx) = &self.notif_tx {
            if let Some(error) = tx.send(notif_result).await.err() {
                error!(
                    "Maker with TradeUUID {} failed in notifying user with handle_taker_offer - {}",
                    self.order.trade_uuid, error
                );
            }
        } else {
            error!(
                "Maker with TradeUUID {} do not have Offer notif_tx registered",
                self.order.trade_uuid
            );
        }
    }
}
