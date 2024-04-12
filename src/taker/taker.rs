use std::path::Path;
use tracing::{debug, error, info, warn};

use strum_macros::{Display, IntoStaticStr};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use uuid::Uuid;

use super::data::TakerData;

use crate::{
    common::{
        error::N3xbError,
        types::{SerdeGenericTrait, SerdeGenericType},
    },
    comms::CommsAccess,
    offer::Offer,
    order::OrderEnvelope,
    peer_msg::PeerEnvelope,
    trade_rsp::{TradeResponse, TradeResponseEnvelope},
};

pub enum TakerNotif {
    TradeRsp(TradeResponseEnvelope),
    Peer(PeerEnvelope),
}

#[derive(Clone)]
pub struct TakerAccess {
    tx: mpsc::Sender<TakerRequest>,
}

impl TakerAccess {
    pub(super) fn new(tx: mpsc::Sender<TakerRequest>) -> Self {
        Self { tx }
    }

    pub async fn take_order(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::SendTakerOffer { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn query_trade_rsp(&self) -> Result<Option<TradeResponseEnvelope>, N3xbError> {
        let (rsp_tx, rsp_rx) =
            oneshot::channel::<Result<Option<TradeResponseEnvelope>, N3xbError>>();
        let request = TakerRequest::QueryTradeRsp { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn send_peer_message(
        &self,
        content: Box<dyn SerdeGenericTrait>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::PeerMessage {
            message: content,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn trade_complete(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::TradeComplete { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn register_notif_tx(
        &self,
        tx: mpsc::Sender<Result<TakerNotif, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::RegisterNotifTx { tx, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn unregister_notif_tx(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::UnregisterNotifTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn shutdown(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::Shutdown { rsp_tx };
        self.tx.send(request).await?; // Shutdown is allowed to fail if already shutdown
        rsp_rx.await?
    }
}

pub(crate) struct Taker {
    tx: mpsc::Sender<TakerRequest>,
    pub(crate) task_handle: tokio::task::JoinHandle<()>,
}

impl Taker {
    const TAKER_REQUEST_CHANNEL_SIZE: usize = 10;

    pub(crate) fn new(
        comms_accessor: CommsAccess,
        order_envelope: OrderEnvelope,
        offer: Offer,
        taker_dir_path: impl AsRef<Path>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<TakerRequest>(Self::TAKER_REQUEST_CHANNEL_SIZE);
        let actor = TakerActor::new(rx, comms_accessor, order_envelope, offer, taker_dir_path);
        let task_handle = tokio::spawn(async move { actor.run().await });
        Self { tx, task_handle }
    }

    pub(crate) fn restore(
        comms_accessor: CommsAccess,
        taker_data_path: impl AsRef<Path>,
    ) -> Result<(Uuid, Self), N3xbError> {
        let (tx, rx) = mpsc::channel::<TakerRequest>(Self::TAKER_REQUEST_CHANNEL_SIZE);
        let (trade_uuid, actor) = TakerActor::restore(rx, comms_accessor, taker_data_path)?;
        let task_handle = tokio::spawn(async move { actor.run().await });
        let taker = Self { tx, task_handle };
        Ok((trade_uuid, taker))
    }

    pub(crate) fn new_accessor(&self) -> TakerAccess {
        TakerAccess::new(self.tx.clone())
    }
}

#[derive(Display, IntoStaticStr)]
pub(super) enum TakerRequest {
    SendTakerOffer {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    QueryTradeRsp {
        rsp_tx: oneshot::Sender<Result<Option<TradeResponseEnvelope>, N3xbError>>,
    },
    PeerMessage {
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    TradeComplete {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterNotifTx {
        tx: mpsc::Sender<Result<TakerNotif, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterNotifTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    Shutdown {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

struct TakerActor {
    rx: mpsc::Receiver<TakerRequest>,
    comms_accessor: CommsAccess,
    data: TakerData,
    notif_tx: Option<mpsc::Sender<Result<TakerNotif, N3xbError>>>,
}

impl TakerActor {
    pub(crate) fn new(
        rx: mpsc::Receiver<TakerRequest>,
        comms_accessor: CommsAccess,
        order_envelope: OrderEnvelope,
        offer: Offer,
        taker_dir_path: impl AsRef<Path>,
    ) -> Self {
        let data = TakerData::new(taker_dir_path, order_envelope, offer);

        TakerActor {
            rx,
            comms_accessor,
            data,
            notif_tx: None,
        }
    }

    pub(crate) fn restore(
        rx: mpsc::Receiver<TakerRequest>,
        comms_accessor: CommsAccess,
        taker_data_path: impl AsRef<Path>,
    ) -> Result<(Uuid, Self), N3xbError> {
        let (trade_uuid, data) = TakerData::restore(taker_data_path)?;

        let actor = TakerActor {
            rx,
            comms_accessor,
            data,
            notif_tx: None,
        };

        Ok((trade_uuid, actor))
    }

    async fn run(mut self) {
        let (tx, mut rx) = mpsc::channel::<PeerEnvelope>(20);

        if let Some(error) = self
            .comms_accessor
            .register_peer_message_tx(self.data.trade_uuid, tx)
            .await
            .err()
        {
            error!(
                "Failed to register Taker for Peer Messages destined for TradeUUID {}. Taker will terminate. Error: {}",
                self.data.trade_uuid,
                error
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
        info!("Taker w/ TradeUUID {} terminating", self.data.trade_uuid);
        self.data.terminate();
    }

    // Top-down Requests Handling

    async fn handle_request(&mut self, request: TakerRequest) -> bool {
        let mut terminate = false;
        debug!(
            "Taker w/ TradeUUID {} handle_request() of type {}",
            self.data.trade_uuid, request
        );

        match request {
            TakerRequest::SendTakerOffer { rsp_tx } => self.send_taker_offer(rsp_tx).await,
            TakerRequest::QueryTradeRsp { rsp_tx } => {
                self.query_trade_rsp(rsp_tx);
            }
            TakerRequest::PeerMessage { message, rsp_tx } => {
                self.send_peer_message(message, rsp_tx).await;
            }
            TakerRequest::TradeComplete { rsp_tx } => {
                self.trade_complete(rsp_tx);
            }
            TakerRequest::RegisterNotifTx { tx, rsp_tx } => {
                self.register_notif_tx(tx, rsp_tx);
            }
            TakerRequest::UnregisterNotifTx { rsp_tx } => {
                self.unregister_notif_tx(rsp_tx);
            }
            TakerRequest::Shutdown { rsp_tx } => {
                self.shutdown(rsp_tx);
                terminate = true;
            }
        }
        terminate
    }

    async fn send_taker_offer(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let order_envelope = self.data.order_envelope();
        let offer = self.data.offer();

        let result = self
            .comms_accessor
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
                self.data.set_offer_event_id(event_id);
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(err) => {
                rsp_tx.send(Err(err)).unwrap(); // oneshot should not fail
            }
        }
    }

    fn query_trade_rsp(
        &mut self,
        rsp_tx: oneshot::Sender<Result<Option<TradeResponseEnvelope>, N3xbError>>,
    ) {
        let trade_rsp = self.data.trade_rsp_envelope();
        rsp_tx.send(Ok(trade_rsp)).unwrap(); // oneshot should not fail
    }

    async fn send_peer_message(
        &mut self,
        message: Box<dyn SerdeGenericTrait>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let order_envelope = self.data.order_envelope();
        let result = self
            .comms_accessor
            .send_trade_engine_specific_message(
                order_envelope.pubkey,
                None,
                order_envelope.event_id,
                order_envelope.order.trade_uuid,
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

    fn register_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<TakerNotif, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} already have notif_tx registered",
                self.data.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = Some(tx);
        rsp_tx.send(result).unwrap();
    }

    fn unregister_notif_tx(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let mut result = Ok(());
        if self.notif_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} does not have notif_tx registered",
                self.data.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    fn check_trade_completed(&self) -> Result<(), N3xbError> {
        if self.data.trade_completed() {
            let error = N3xbError::Simple(format!(
                "Taker w/ TradeUUID {} already marked as Trade Complete",
                self.data.trade_uuid
            ));
            Err(error) // oneshot should not fail
        } else {
            Ok(())
        }
    }

    fn trade_complete(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        // TODO: What else to do for Trade Complete?
        self.data.set_trade_completed(true);
        rsp_tx.send(Ok(())).unwrap();
    }

    fn shutdown(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        rsp_tx.send(Ok(())).unwrap();
    }

    // Bottom-up Peer Message Handling

    async fn handle_peer_message(&mut self, peer_envelope: PeerEnvelope) {
        debug!(
            "Taker w/ TradeUUID {} handle_peer_message() from pubkey {}, of event id {}, type {:?}",
            self.data.trade_uuid,
            peer_envelope.pubkey.to_string(),
            peer_envelope.event_id.to_string(),
            peer_envelope.message_type
        );

        match peer_envelope.message_type {
            SerdeGenericType::TradeResponse => {
                let trade_rsp = peer_envelope.message
                    .downcast_ref::<TradeResponse>()
                    .expect(
                        &format!(
                            "Taker w/ TradeUUID {} received peer message of SerdeGenericType::TakerOffer, but failed to downcast message into Offer",
                            self.data.trade_uuid
                        )
                    )
                    .to_owned();
                let trade_rsp_envelope = TradeResponseEnvelope {
                    pubkey: peer_envelope.pubkey,
                    urls: peer_envelope.urls,
                    event_id: peer_envelope.event_id,
                    trade_rsp: trade_rsp,
                    _private: (),
                };
                self.handle_trade_response(trade_rsp_envelope).await;
            }

            SerdeGenericType::TakerOffer => {
                error!(
                    "Taker w/ TradeUUID {} received unexpected TakerOffer message",
                    self.data.trade_uuid
                );
            }

            SerdeGenericType::TradeEngineSpecific => {
                self.handle_engine_specific_peer_message(peer_envelope)
                    .await;
            }
        }
    }

    async fn handle_trade_response(&mut self, trade_rsp_envelope: TradeResponseEnvelope) {
        let mut notif_result: Result<TakerNotif, N3xbError> =
            Ok(TakerNotif::TradeRsp(trade_rsp_envelope.clone()));

        let order_envelope = self.data.order_envelope();
        let offer_event_id = self.data.offer_event_id().expect(&format!(
            "Taker w/ TradeUUID {} received TradeResponse message before Taker Offer has been sent",
            self.data.trade_uuid
        ));

        if trade_rsp_envelope.pubkey != order_envelope.pubkey {
            notif_result = Err(
                N3xbError::Simple(
                    format!(
                        "Taker w/ TradeUUID {} received TradeResponse message with unexpected pubkey. Expected pubkey: {}, Received pubkey: {}",
                        self.data.trade_uuid,
                        order_envelope.pubkey,
                        trade_rsp_envelope.pubkey
                    )
                )
            );
        } else if let Some(existing_trade_rsp_envelope) = &self.data.trade_rsp_envelope() {
            notif_result = Err(
                N3xbError::Simple(
                    format!(
                        "Taker w/ TradeUUID {} received duplicate TradeResponse message. Previous TradeResponse: {:?}, New TradeResponse: {:?}",
                        self.data.trade_uuid,
                        existing_trade_rsp_envelope,
                        trade_rsp_envelope
                    )
                )
            );
        } else if trade_rsp_envelope.trade_rsp.offer_event_id != offer_event_id {
            notif_result = Err(
                N3xbError::Simple(
                    format!(
                        "Taker w/ TradeUUID {} received TradeResponse message with unexpected Offer Event ID. Expected EventId: {:?}, Received EventId: {:?}",
                        self.data.trade_uuid,
                        offer_event_id,
                        trade_rsp_envelope.trade_rsp.offer_event_id
                    )
                )
            );
        } else {
            self.data.set_trade_rsp_envelope(trade_rsp_envelope);
        }

        // Notify user of new Trade Response recieved
        if let Some(tx) = &self.notif_tx {
            if let Some(error) = tx.send(notif_result).await.err() {
                error!(
                    "Taker w/ TradeUUID {} failed in notifying user with handle_trade_response - {}",
                    self.data.trade_uuid,
                    error
                );
            }
        } else {
            warn!(
                "Taker w/ TradeUUID {} do not have Offer notif_tx registered",
                self.data.trade_uuid
            );
        }
    }

    async fn handle_engine_specific_peer_message(&mut self, envelope: PeerEnvelope) {
        let order_envelope = self.data.order_envelope();

        // Verify peer message is signed by the expected pubkey before passing to Trade Engine
        if envelope.pubkey != order_envelope.pubkey {
            error!(
                "Taker w/ TradeUUID {} received TradeEngineSpecific message with unexpected pubkey. Expected pubkey: {}, Received pubkey: {}",
                self.data.trade_uuid,
                order_envelope.pubkey,
                envelope.pubkey
            );
            return;
        }

        // Let the Trade Engine / user to do the downcasting. Pass the SerdeGeneric message up as is
        if let Some(tx) = &self.notif_tx {
            if let Some(error) = tx.send(Ok(TakerNotif::Peer(envelope))).await.err() {
                error!(
                    "Taker w/ TradeUUID {} failed in notifying user with handle_peer_message - {}",
                    self.data.trade_uuid, error
                );
            }
        } else {
            warn!(
                "Taker w/ TradeUUID {} do not have notif_tx registered",
                self.data.trade_uuid
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
