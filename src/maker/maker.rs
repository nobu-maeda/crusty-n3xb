use std::{collections::HashMap, path::Path};
use strum_macros::{Display, IntoStaticStr};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use tokio::{
    select,
    sync::{mpsc, oneshot},
};

use super::data::MakerData;

use crate::{
    common::{
        error::{N3xbError, OfferInvalidReason},
        types::{EventIdString, SerdeGenericTrait, SerdeGenericType},
    },
    comms::CommsAccess,
    offer::{Offer, OfferEnvelope},
    order::Order,
    peer_msg::PeerEnvelope,
    trade_rsp::{TradeResponse, TradeResponseBuilder, TradeResponseStatus},
};

pub enum MakerNotif {
    Offer(OfferEnvelope),
    Peer(PeerEnvelope),
}

#[derive(Clone)]
pub struct MakerAccess {
    tx: mpsc::Sender<MakerRequest>,
}

impl MakerAccess {
    pub(super) fn new(tx: mpsc::Sender<MakerRequest>) -> Self {
        Self { tx }
    }

    pub async fn post_new_order(&self) -> Result<(), N3xbError> {
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

    pub async fn cancel_order(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::CancelOrder { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

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

    pub async fn trade_complete(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::TradeComplete { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn register_notif_tx(
        &self,
        tx: mpsc::Sender<Result<MakerNotif, N3xbError>>,
    ) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::RegisterNotifTx { tx, rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn unregister_notif_tx(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::UnregisterNotifTx { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn shutdown(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::Shutdown { rsp_tx };
        self.tx.send(request).await?; // Shutdown is allowed to fail if already shutdown
        rsp_rx.await?
    }
}

pub(crate) struct Maker {
    tx: mpsc::Sender<MakerRequest>,
    pub(crate) task_handle: tokio::task::JoinHandle<()>,
}

impl Maker {
    const MAKER_REQUEST_CHANNEL_SIZE: usize = 10;

    pub(crate) fn new(
        comms_accessor: CommsAccess,
        order: Order,
        maker_dir_path: impl AsRef<Path>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<MakerRequest>(Self::MAKER_REQUEST_CHANNEL_SIZE);
        let actor = MakerActor::new(rx, comms_accessor, order, maker_dir_path);
        let task_handle = tokio::spawn(async move { actor.run().await });
        Self { tx, task_handle }
    }

    pub(crate) fn restore(
        comms_accessor: CommsAccess,
        maker_data_path: impl AsRef<Path>,
    ) -> Result<(Uuid, Self), N3xbError> {
        let (tx, rx) = mpsc::channel::<MakerRequest>(Self::MAKER_REQUEST_CHANNEL_SIZE);
        let (trade_uuid, actor) = MakerActor::restore(rx, comms_accessor, maker_data_path)?;
        let task_handle = tokio::spawn(async move { actor.run().await });
        let maker = Self { tx, task_handle };
        Ok((trade_uuid, maker))
    }

    pub(crate) fn new_accessor(&self) -> MakerAccess {
        MakerAccess::new(self.tx.clone())
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
    RegisterNotifTx {
        tx: mpsc::Sender<Result<MakerNotif, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    UnregisterNotifTx {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    Shutdown {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
}

struct MakerActor {
    rx: mpsc::Receiver<MakerRequest>,
    comms_accessor: CommsAccess,
    data: MakerData,
    notif_tx: Option<mpsc::Sender<Result<MakerNotif, N3xbError>>>,
}

impl MakerActor {
    pub(crate) fn new(
        rx: mpsc::Receiver<MakerRequest>,
        comms_accessor: CommsAccess,
        order: Order,
        maker_dir_path: impl AsRef<Path>,
    ) -> Self {
        let data = MakerData::new(maker_dir_path, order, true);

        MakerActor {
            rx,
            comms_accessor,
            data,
            notif_tx: None,
        }
    }

    pub(crate) fn restore(
        rx: mpsc::Receiver<MakerRequest>,
        comms_accessor: CommsAccess,
        maker_data_path: impl AsRef<Path>,
    ) -> Result<(Uuid, Self), N3xbError> {
        let (trade_uuid, data) = MakerData::restore(maker_data_path)?;

        let actor = MakerActor {
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
                "Failed to register Maker for Peer Messages destined for TradeUUID {}. Maker will terminate. Error: {}",
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
        info!("Maker w/ TradeUUID {} terminating", self.data.trade_uuid);
        self.data.terminate();
    }

    // Top-down Request Handling

    async fn handle_request(&mut self, request: MakerRequest) -> bool {
        let mut terminate = false;

        debug!(
            "Maker w/ TradeUUID {} handle_request() of type {}",
            self.data.trade_uuid, request
        );

        match request {
            MakerRequest::SendMakerOrder { rsp_tx } => self.send_maker_order(rsp_tx).await,
            MakerRequest::QueryOffers { rsp_tx } => self.query_offers(rsp_tx),
            MakerRequest::QueryOffer { event_id, rsp_tx } => {
                self.query_offer(event_id, rsp_tx);
            }
            MakerRequest::AcceptOffer { trade_rsp, rsp_tx } => {
                self.accept_offer(trade_rsp, rsp_tx).await;
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
            }
            MakerRequest::RegisterNotifTx { tx, rsp_tx } => {
                self.register_notif_tx(tx, rsp_tx);
            }
            MakerRequest::UnregisterNotifTx { rsp_tx } => {
                self.unregister_notif_tx(rsp_tx);
            }
            MakerRequest::Shutdown { rsp_tx } => {
                self.shutdown(rsp_tx);
                terminate = true;
            }
        }
        terminate
    }

    async fn send_maker_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let order = self.data.order();
        let result = self.comms_accessor.send_maker_order_note(order).await;
        match result {
            Ok(order_envelope) => {
                self.data
                    .update_maker_order(order_envelope.event_id, order_envelope.urls);
                rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
            }
            Err(error) => {
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            }
        }
    }

    fn query_offers(&mut self, rsp_tx: oneshot::Sender<HashMap<EventIdString, OfferEnvelope>>) {
        rsp_tx.send(self.data.offer_envelopes()).unwrap(); // oneshot should not fail
    }

    fn query_offer(
        &mut self,
        event_id: EventIdString,
        rsp_tx: oneshot::Sender<Option<OfferEnvelope>>,
    ) {
        let offer = self.data.offer_envelopes().get(&event_id).cloned();
        rsp_tx.send(offer).unwrap(); // oneshot should not fail
    }

    async fn accept_offer(
        &mut self,
        trade_rsp: TradeResponse,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        if let Some(event_id) = self.data.accepted_offer_event_id() {
            let error = N3xbError::Simple(
                format!(
                    "Maker w/ TradeUUID {} should not have already accepted an Offer. Prev Offer event ID {}, New Offer event ID {}",
                    self.data.trade_uuid,
                    event_id,
                    trade_rsp.offer_event_id
                )
            );
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let accepted_offer_event_id = trade_rsp.offer_event_id.clone();
        self.data
            .set_accepted_offer_event_id(accepted_offer_event_id.clone());

        let pubkey = match self.data.offer_envelopes().get(&accepted_offer_event_id) {
            Some(offer_envelope) => offer_envelope.pubkey.clone(),
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected, but does not contain accepted Offer {}",
                    self.data.trade_uuid, accepted_offer_event_id
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let maker_order_note_id = match self.data.order_event_id() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(
                    format!(
                        "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                        self.data.trade_uuid
                    )
                );
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        // Send Trade Response Pending to all other Offers
        for offer_envelope in self.data.offer_envelopes().values() {
            let offer_event_id = offer_envelope.event_id.clone();
            if offer_event_id == accepted_offer_event_id {
                continue;
            } else {
                warn!(
                    "Maker w/ TradeUUID {} has other outstanding offers, but no explicit rejection sent to Takers",
                    self.data.trade_uuid
                );
            }

            let offer_envelope = offer_envelope.clone();

            if let Some(reject_err) = self
                .reject_taker_offer(offer_envelope, OfferInvalidReason::PendingAnother)
                .await
                .err()
            {
                error!(
                    "Maker w/ TradeUUID {} rejected Offer with Event ID {} but with error - {}",
                    self.data.order().trade_uuid.clone(),
                    offer_event_id,
                    reject_err
                )
            }
        }

        let trade_rsp_clone = trade_rsp.clone();

        let result = self
            .comms_accessor
            .send_trade_response(
                pubkey,
                Some(accepted_offer_event_id),
                maker_order_note_id.clone(),
                self.data.trade_uuid,
                trade_rsp_clone,
            )
            .await;

        match result {
            Ok(event_id) => {
                self.data.set_trade_rsp(trade_rsp, event_id);
            }
            Err(error) => {
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        }

        // Delete Order Note
        let result = self
            .comms_accessor
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

    async fn cancel_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let maker_order_note_id = match self.data.order_event_id() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(
                    format!(
                        "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                        self.data.trade_uuid
                    )
                );
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        // Send Trade Response Cancelled to all Offers received so far
        for offer_envelope in self.data.offer_envelopes().values() {
            warn!(
                "Maker w/ TradeUUID {} has outstanding offers, but no explicit cancellation sent to Takers",
                self.data.trade_uuid
            );

            if let Some(reject_err) = self
                .reject_taker_offer(offer_envelope.clone(), OfferInvalidReason::Cancelled)
                .await
                .err()
            {
                error!(
                    "Maker w/ TradeUUID {} rejected Offer with Event ID {} but with error - {}",
                    self.data.trade_uuid, offer_envelope.event_id, reject_err
                );
            }
        }

        // Delete Order Note
        let result = self
            .comms_accessor
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
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        let accepted_offer_event_id = match self.data.accepted_offer_event_id() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected to already have accepted an Offer",
                    self.data.trade_uuid
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let pubkey = match self.data.offer_envelopes().get(&accepted_offer_event_id) {
            Some(offer_envelope) => offer_envelope.pubkey.clone(),
            None => {
                let error = N3xbError::Simple(format!(
                    "Maker w/ TradeUUID {} expected, but does not contain accepted Offer {}",
                    self.data.trade_uuid, accepted_offer_event_id
                ));
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let maker_order_note_id = match self.data.order_event_id() {
            Some(event_id) => event_id,
            None => {
                let error = N3xbError::Simple(
                    format!(
                        "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                        self.data.trade_uuid
                    )
                );
                rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
                return;
            }
        };

        let result = self
            .comms_accessor
            .send_trade_engine_specific_message(
                pubkey,
                None,
                maker_order_note_id,
                self.data.trade_uuid,
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

    fn check_trade_completed(&self) -> Result<(), N3xbError> {
        if self.data.trade_completed() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} already marked as Trade Complete",
                self.data.trade_uuid
            ));
            Err(error) // oneshot should not fail
        } else {
            Ok(())
        }
    }

    async fn trade_complete(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        if let Some(error) = self.check_trade_completed().err() {
            rsp_tx.send(Err(error)).unwrap(); // oneshot should not fail
            return;
        }

        // TODO: What else to do for Trade Complete?
        self.data.set_trade_completed(true);
        rsp_tx.send(Ok(())).unwrap(); // oneshot should not fail
    }

    fn register_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<MakerNotif, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Maker w/ TradeUUID {} already have notif_tx registered",
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
                "Maker w/ TradeUUID {} expected to already have notif_tx registered",
                self.data.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = None;
        rsp_tx.send(result).unwrap();
    }

    fn shutdown(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        rsp_tx.send(Ok(())).unwrap();
    }

    // Bottom-up Peer Message Handling

    async fn handle_peer_message(&mut self, peer_envelope: PeerEnvelope) {
        debug!(
            "Maker w/ TradeUUID {} handle_peer_message() from pubkey {}, of event id {}, type {:?}",
            self.data.trade_uuid,
            peer_envelope.pubkey.to_string(),
            peer_envelope.event_id.to_string(),
            peer_envelope.message_type
        );

        match peer_envelope.message_type {
            SerdeGenericType::TakerOffer => {
                let offer = peer_envelope.message
                    .downcast_ref::<Offer>()
                    .expect(
                        &format!(
                            "Maker w/ TradeUUID {} received peer message of SerdeGenericType::TakerOffer, but failed to downcast message into Offer",
                            self.data.trade_uuid
                        )
                    )
                    .to_owned();
                let offer_envelope = OfferEnvelope {
                    pubkey: peer_envelope.pubkey,
                    urls: peer_envelope.urls,
                    event_id: peer_envelope.event_id,
                    offer,
                    _private: (),
                };
                self.handle_taker_offer(offer_envelope).await;
            }

            SerdeGenericType::TradeResponse => {
                error!(
                    "Maker w/ TradeUUID {} received unexpected TradeResponse message",
                    self.data.trade_uuid
                );
            }

            SerdeGenericType::TradeEngineSpecific => {
                self.handle_engine_specific_peer_message(peer_envelope)
                    .await;
            }
        }
    }

    async fn handle_taker_offer(&mut self, offer_envelope: OfferEnvelope) {
        let mut notif_result: Result<MakerNotif, N3xbError> =
            Ok(MakerNotif::Offer(offer_envelope.clone()));

        let reason = if self.data.accepted_offer_event_id().is_some() {
            Some(OfferInvalidReason::PendingAnother)
        } else if self
            .data
            .offer_envelopes()
            .contains_key(&offer_envelope.event_id)
        {
            Some(OfferInvalidReason::DuplicateOffer)
        } else if let Some(reason) = offer_envelope
            .offer
            .validate_against(&self.data.order())
            .err()
        {
            Some(reason)
        } else {
            self.data
                .insert_offer_envelope(offer_envelope.event_id.clone(), offer_envelope.clone());
            None
        };

        let offer_envelope_clone = offer_envelope.clone();

        let accepted_offer_string =
            if let Some(offer_event_id) = self.data.accepted_offer_event_id() {
                offer_event_id.to_string()
            } else {
                "N/A".to_string()
            };

        debug!("Maker w/ TradeUUID {} handling Taker Offer with Event ID {} Accepted ID? {} - reason: {:?}", 
                 self.data.trade_uuid, offer_envelope.event_id, accepted_offer_string, reason);

        if let Some(reason) = reason {
            notif_result = Err(N3xbError::InvalidOffer(reason.clone()));
            if let Some(reject_err) = self
                .reject_taker_offer(offer_envelope_clone, reason)
                .await
                .err()
            {
                error!(
                    "Maker w/ TradeUUID {} rejected Offer with Event ID {} but with error - {}",
                    self.data.trade_uuid, offer_envelope.event_id, reject_err
                );
            }
            if self.data.reject_invalid_offers_silently() {
                return;
            }
        }

        // Notify user of new Offer recieved
        if let Some(tx) = &self.notif_tx {
            if let Some(error) = tx.send(notif_result).await.err() {
                error!(
                    "Maker w/ TradeUUID {} failed in notifying user with handle_taker_offer - {}",
                    self.data.trade_uuid, error
                );
            }
        } else {
            warn!(
                "Maker w/ TradeUUID {} do not have notif_tx registered",
                self.data.trade_uuid
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
        let maker_order_note_id = match self.data.order_event_id() {
            Some(event_id) => event_id,
            None => {
                reject_result = Err(
                    N3xbError::Simple(
                        format!(
                            "Maker w/ TradeUUID {} expected to already have sent Maker Order Note and receive Event ID",
                            self.data.trade_uuid
                        )
                    )
                );
                "".to_string()
            }
        };

        let trade_rsp = TradeResponseBuilder::new()
            .offer_event_id(offer_event_id)
            .trade_response(TradeResponseStatus::Rejected)
            .reject_reason(reason.clone())
            .build()
            .unwrap();

        self.comms_accessor
            .send_trade_response(
                pubkey,
                Some(offer_envelope.event_id.clone()),
                maker_order_note_id,
                self.data.trade_uuid.clone(),
                trade_rsp,
            )
            .await?;

        reject_result
    }

    async fn handle_engine_specific_peer_message(&mut self, envelope: PeerEnvelope) {
        // Verify peer message is signed by the expected pubkey before passing to Trade Engine
        let expected_pubkey =
            if let Some(accepted_offer_event_id) = self.data.accepted_offer_event_id() {
                match self.data.offer_envelopes().get(&accepted_offer_event_id) {
                    Some(offer_envelope) => offer_envelope.pubkey.clone(),
                    None => {
                        error!(
                            "Maker w/ TradeUUID {} expected to contain accepted Offer {}",
                            self.data.trade_uuid, accepted_offer_event_id
                        );
                        return;
                    }
                }
            } else {
                error!(
                    "Maker w/ TradeUUID {} expected to already have accepted an Offer",
                    self.data.trade_uuid
                );
                return;
            };

        if envelope.pubkey != expected_pubkey {
            error!(
                "Maker w/ TradeUUID {} received TradeEngineSpecific message from unexpected pubkey {}",
                self.data.trade_uuid,
                envelope.pubkey
            );
            return;
        }

        // Let the Trade Engine / user to do the downcasting. Pass the SerdeGeneric message up as is
        if let Some(tx) = &self.notif_tx {
            if let Some(error) = tx.send(Ok(MakerNotif::Peer(envelope))).await.err() {
                error!(
                    "Maker w/ TradeUUID {} failed in notifying user with handle_peer_message - {}",
                    self.data.trade_uuid, error
                );
            }
        } else {
            warn!(
                "Maker w/ TradeUUID {} do not have notif_tx registered",
                self.data.trade_uuid
            );
        }
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
