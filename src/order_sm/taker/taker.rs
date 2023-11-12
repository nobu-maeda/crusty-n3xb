use log::{debug, error};

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
    offer::Offer,
    order::OrderEnvelope,
    trade_rsp::{TradeResponse, TradeResponseEnvelope},
};

pub struct Taker {
    tx: mpsc::Sender<TakerRequest>,
}

impl Taker {
    pub(super) async fn new(tx: mpsc::Sender<TakerRequest>) -> Self {
        Self { tx }
    }

    pub(crate) async fn send_taker_offer(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::SendTakerOffer { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn register_trade_notif_tx(
        &self,
        tx: mpsc::Sender<Result<TradeResponseEnvelope, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::RegisterTradeNotifTx { tx, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn unregister_trade_notif_tx(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::UnregisterTradeNotifTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

pub(crate) struct TakerEngine {
    tx: mpsc::Sender<TakerRequest>,
}

impl TakerEngine {
    const TAKER_REQUEST_CHANNEL_SIZE: usize = 2;

    pub(crate) async fn new(
        interfacer_handle: InterfacerHandle,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<TakerRequest>(Self::TAKER_REQUEST_CHANNEL_SIZE);
        let mut actor = TakerActor::new(rx, interfacer_handle, order_envelope, offer).await;
        tokio::spawn(async move { actor.run().await });
        Self { tx }
    }

    // Interfacer Handle

    pub(crate) async fn new_handle(&self) -> Taker {
        Taker::new(self.tx.clone()).await
    }
}

#[derive(Debug)]
pub(super) enum TakerRequest {
    SendTakerOffer {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterTradeNotifTx {
        tx: mpsc::Sender<Result<TradeResponseEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterTradeNotifTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

struct TakerActor {
    rx: mpsc::Receiver<TakerRequest>,
    interfacer_handle: InterfacerHandle,
    order_envelope: OrderEnvelope,
    offer: Offer,
    offer_event_id: Option<EventIdString>,
    trade_rsp_envelope: Option<TradeResponseEnvelope>,
    notif_tx: Option<mpsc::Sender<Result<TradeResponseEnvelope, N3xbError>>>,
}

impl TakerActor {
    pub(crate) async fn new(
        rx: mpsc::Receiver<TakerRequest>,
        interfacer_handle: InterfacerHandle,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Self {
        TakerActor {
            rx,
            interfacer_handle,
            order_envelope,
            offer,
            offer_event_id: None,
            trade_rsp_envelope: None,
            notif_tx: None,
        }
    }

    async fn run(&mut self) {
        let (tx, mut rx) = mpsc::channel::<PeerEnvelope>(20);
        let trade_uuid = self.order_envelope.order.trade_uuid;

        if let Some(error) = self
            .interfacer_handle
            .register_peer_message_tx(trade_uuid, tx)
            .await
            .err()
        {
            error!(
                "Failed to register Taker for Peer Messages destined for TradeUUID {}. Taker will terminate. Error: {}",
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

    async fn handle_request(&mut self, request: TakerRequest) {
        debug!(
            "Taker w/ TradeUUID {} handle_request() of type {:?}",
            self.order_envelope.order.trade_uuid, request
        );

        match request {
            TakerRequest::SendTakerOffer { rsp_tx } => self.send_taker_offer(rsp_tx).await,
            TakerRequest::RegisterTradeNotifTx { tx, rsp_tx } => {
                self.register_trade_notif_tx(tx, rsp_tx).await
            }
            TakerRequest::UnregisterTradeNotifTx { rsp_tx } => {
                self.unregister_trade_notif_tx(rsp_tx).await
            }
        }
    }

    async fn send_taker_offer(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order_envelope = self.order_envelope.clone();
        let offer = self.offer.clone();

        let result = self
            .interfacer_handle
            .send_taker_offer_message(
                order_envelope.pubkey,
                Some(order_envelope.event_id.clone()),
                order_envelope.event_id,
                order_envelope.order.trade_uuid,
                offer,
            )
            .await;

        match result {
            Ok(event_id) => {
                self.offer_event_id = Some(event_id);
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(err) => {
                rsp_tx.send(Err(err)).unwrap(); // oneshot should not fail
            }
        }
    }

    async fn register_trade_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<TradeResponseEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} already have notif_tx registered",
                self.order_envelope.order.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = Some(tx);
        rsp_tx.send(result).unwrap();
    }

    async fn unregister_trade_notif_tx(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let mut result = Ok(());
        if self.notif_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} does not have notif_tx registered",
                self.order_envelope.order.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    async fn handle_peer_message(&mut self, peer_envelope: PeerEnvelope) {
        debug!(
            "Taker w/ TradeUUID {} handle_peer_message() from pubkey {}, of event id {}, type {:?}",
            self.order_envelope.order.trade_uuid,
            peer_envelope.pubkey.to_string(),
            peer_envelope.event_id.to_string(),
            peer_envelope.message_type
        );

        match peer_envelope.message_type {
            SerdeGenericType::TradeResponse => {
                let trade_rsp = peer_envelope.message.downcast_ref::<TradeResponse>().expect(&format!("Taker w/ TradeUUID {} received peer message of SerdeGenericType::TakerOffer, but failed to downcast message into Offer", self.order_envelope.order.trade_uuid)).to_owned();
                let trade_rsp_envelope = TradeResponseEnvelope {
                    pubkey: peer_envelope.pubkey,
                    event_id: peer_envelope.event_id,
                    trade_rsp: trade_rsp,
                    _private: (),
                };
                self.handle_trade_response(trade_rsp_envelope).await;
            }

            SerdeGenericType::TakerOffer => {
                error!(
                    "Taker w/ TradeUUID {} received unexpected TakerOffer message",
                    self.order_envelope.order.trade_uuid
                );
            }

            SerdeGenericType::TradeEngineSpecific => {
                todo!();
            }
        }
    }

    async fn handle_trade_response(&mut self, trade_rsp_envelope: TradeResponseEnvelope) {
        let mut notif_result: Result<TradeResponseEnvelope, N3xbError> =
            Ok(trade_rsp_envelope.clone());

        if let Some(existing_trade_rsp_envelope) = &self.trade_rsp_envelope {
            notif_result = Err(N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} received duplicate TradeResponse message. Previous TradeResponse: {:?}, New TradeResponse: {:?}",
                self.order_envelope.order.trade_uuid, existing_trade_rsp_envelope, trade_rsp_envelope
            )));
        } else if trade_rsp_envelope.trade_rsp.offer_event_id
            != self.offer_event_id.clone().unwrap()
        {
            notif_result = Err(N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} received TradeResponse message with unexpected Offer Event ID. Expected EventId: {:?}, Received EventId: {:?}",
                self.order_envelope.order.trade_uuid, self.offer_event_id, trade_rsp_envelope.trade_rsp.offer_event_id
            )));
        } else {
            self.trade_rsp_envelope = Some(trade_rsp_envelope.clone());
        }

        // Notify user of new Trade Response recieved
        if let Some(tx) = &self.notif_tx {
            if let Some(error) = tx.send(notif_result).await.err() {
                error!(
                    "Taker w/ TradeUUID {} failed in notifying user with handle_trade_response - {}",
                    self.order_envelope.order.trade_uuid, error
                );
            }
        } else {
            error!(
                "Taker w/ TradeUUID {} do not have Offer notif_tx registered",
                self.order_envelope.order.trade_uuid
            );
        }
    }
}
