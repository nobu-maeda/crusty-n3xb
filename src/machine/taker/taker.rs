use std::marker::PhantomData;

use log::{debug, error, info, warn};

use strum_macros::{Display, IntoStaticStr};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};

use crate::{
    common::{
        error::N3xbError,
        types::{EventIdString, SerdeGenericTrait, SerdeGenericType},
    },
    communicator::CommunicatorAccess,
    offer::Offer,
    order::OrderEnvelope,
    peer_msg::PeerEnvelope,
    trade_rsp::{TradeResponse, TradeResponseEnvelope},
};

pub struct New;
pub struct Trading;

pub struct TakerAccess<State = New> {
    tx: mpsc::Sender<TakerRequest>,
    state: PhantomData<State>,
}

impl TakerAccess<New> {
    pub async fn take_order(self) -> Result<TakerAccess<Trading>, N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::SendTakerOffer { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()?;

        Ok(TakerAccess {
            tx: self.tx,
            state: PhantomData,
        })
    }
}

impl TakerAccess<Trading> {
    pub async fn send_peer_message(
        &self,
        _content: Box<dyn SerdeGenericTrait>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::PeerMessage {
            message: _content,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn trade_complete(self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::TradeComplete { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

impl<State> TakerAccess<State> {
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

    pub async fn register_peer_notif_tx(
        &self,
        tx: mpsc::Sender<Result<PeerEnvelope, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::RegisterPeerNotifTx { tx, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn unregister_peer_notif_tx(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::UnregisterPeerNotifTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }
}

impl TakerAccess {
    pub(super) async fn new(tx: mpsc::Sender<TakerRequest>) -> Self {
        Self {
            tx,
            state: PhantomData,
        }
    }
}

pub(crate) struct Taker {
    tx: mpsc::Sender<TakerRequest>,
    pub task_handle: tokio::task::JoinHandle<()>,
}

impl Taker {
    const TAKER_REQUEST_CHANNEL_SIZE: usize = 10;

    pub(crate) async fn new(
        communicator_accessor: CommunicatorAccess,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<TakerRequest>(Self::TAKER_REQUEST_CHANNEL_SIZE);
        let mut actor = TakerActor::new(rx, communicator_accessor, order_envelope, offer).await;
        let task_handle = tokio::spawn(async move { actor.run().await });
        Self { tx, task_handle }
    }

    // Communicator Handle

    pub(crate) async fn new_accessor(&self) -> TakerAccess {
        TakerAccess::new(self.tx.clone()).await
    }
}

#[derive(Display, IntoStaticStr)]
pub(super) enum TakerRequest {
    SendTakerOffer {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    PeerMessage {
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    TradeComplete {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterTradeNotifTx {
        tx: mpsc::Sender<Result<TradeResponseEnvelope, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterTradeNotifTx {
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

struct TakerActor {
    rx: mpsc::Receiver<TakerRequest>,
    communicator_accessor: CommunicatorAccess,
    order_envelope: OrderEnvelope,
    offer: Offer,
    offer_event_id: Option<EventIdString>,
    trade_rsp_envelope: Option<TradeResponseEnvelope>,
    trade_notif_tx: Option<mpsc::Sender<Result<TradeResponseEnvelope, N3xbError>>>,
    peer_notif_tx: Option<mpsc::Sender<Result<PeerEnvelope, N3xbError>>>,
}

impl TakerActor {
    pub(crate) async fn new(
        rx: mpsc::Receiver<TakerRequest>,
        communicator_accessor: CommunicatorAccess,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Self {
        TakerActor {
            rx,
            communicator_accessor,
            order_envelope,
            offer,
            offer_event_id: None,
            trade_rsp_envelope: None,
            trade_notif_tx: None,
            peer_notif_tx: None,
        }
    }

    async fn run(&mut self) {
        let (tx, mut rx) = mpsc::channel::<PeerEnvelope>(20);
        let trade_uuid = self.order_envelope.order.trade_uuid;

        if let Some(error) = self
            .communicator_accessor
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
        info!("Taker w/ TradeUUID {} terminating", trade_uuid)
    }

    // Top-down Requests Handling

    async fn handle_request(&mut self, request: TakerRequest) -> bool {
        let mut terminate = false;
        debug!(
            "Taker w/ TradeUUID {} handle_request() of type {}",
            self.order_envelope.order.trade_uuid, request
        );

        match request {
            TakerRequest::SendTakerOffer { rsp_tx } => self.send_taker_offer(rsp_tx).await,
            TakerRequest::PeerMessage { message, rsp_tx } => {
                self.send_peer_message(message, rsp_tx).await;
            }
            TakerRequest::TradeComplete { rsp_tx } => {
                self.trade_complete(rsp_tx).await;
                terminate = true;
            }
            TakerRequest::RegisterTradeNotifTx { tx, rsp_tx } => {
                self.register_trade_notif_tx(tx, rsp_tx).await
            }
            TakerRequest::UnregisterTradeNotifTx { rsp_tx } => {
                self.unregister_trade_notif_tx(rsp_tx).await
            }
            TakerRequest::RegisterPeerNotifTx { tx, rsp_tx } => {
                self.register_peer_notif_tx(tx, rsp_tx).await
            }
            TakerRequest::UnregisterPeerNotifTx { rsp_tx } => {
                self.unregister_peer_notif_tx(rsp_tx).await
            }
        }
        terminate
    }

    async fn send_taker_offer(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order_envelope = self.order_envelope.clone();
        let offer = self.offer.clone();

        let result = self
            .communicator_accessor
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

    async fn send_peer_message(
        &mut self,
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let result = self
            .communicator_accessor
            .send_trade_engine_specific_message(
                self.order_envelope.pubkey,
                None,
                self.order_envelope.event_id.clone(),
                self.order_envelope.order.trade_uuid,
                message,
            )
            .await;

        match result {
            Ok(_) => {
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
        if self.trade_notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} already have trade_notif_tx registered",
                self.order_envelope.order.trade_uuid
            ));
            result = Err(error);
        }
        self.trade_notif_tx = Some(tx);
        rsp_tx.send(result).unwrap();
    }

    async fn unregister_trade_notif_tx(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let mut result = Ok(());
        if self.trade_notif_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} does not have trade_notif_tx registered",
                self.order_envelope.order.trade_uuid
            ));
            result = Err(error);
        }
        self.trade_notif_tx = None;
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
                "Taker w/ TradeUUID {} already have peer_notif_tx registered",
                self.order_envelope.order.trade_uuid
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
                "Taker w/ TradeUUID {} does not have peer_notif_tx registered",
                self.order_envelope.order.trade_uuid
            ));
            result = Err(error);
        }
        self.peer_notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    async fn trade_complete(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        // TODO: What else to do for Trade Complete?
        rsp_tx.send(Ok(())).unwrap();
    }

    // Bottom-up Peer Message Handling

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
                // Let the Trade Engine / user to do the downcasting. Pass the SerdeGeneric message up as is
                if let Some(tx) = &self.peer_notif_tx {
                    if let Some(error) = tx.send(Ok(peer_envelope)).await.err() {
                        error!(
                            "Taker w/ TradeUUID {} failed in notifying user with handle_peer_message - {}",
                            self.order_envelope.order.trade_uuid, error
                        );
                    }
                } else {
                    warn!(
                        "Taker w/ TradeUUID {} do not have Offer peer_notif_tx registered",
                        self.order_envelope.order.trade_uuid
                    );
                }
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
        if let Some(tx) = &self.trade_notif_tx {
            if let Some(error) = tx.send(notif_result).await.err() {
                error!(
                    "Taker w/ TradeUUID {} failed in notifying user with handle_trade_response - {}",
                    self.order_envelope.order.trade_uuid, error
                );
            }
        } else {
            warn!(
                "Taker w/ TradeUUID {} do not have Offer trade_notif_tx registered",
                self.order_envelope.order.trade_uuid
            );
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: A lot to mock. Postponing this

    // #[tokio::test]
    // async fn test_handle_trade_response_notif() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_handle_trade_response_duplicated() {
    //     todo!();
    // }

    // #[tokio::test]
    // async fn test_handle_trade_response_offer_id_mismatch() {
    //     todo!();
    // }
}
