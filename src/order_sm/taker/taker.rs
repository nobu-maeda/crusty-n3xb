use tokio::{
    select,
    sync::{mpsc, oneshot},
};

use crate::{
    common::{error::N3xbError, types::EventIdString},
    interfacer::InterfacerHandle,
    offer::Offer,
    order::OrderEnvelope,
    trade_rsp::TradeResponseEnvelope,
};

pub struct Taker {
    tx: mpsc::Sender<TakerRequest>,
}

impl Taker {
    pub(super) async fn new(tx: mpsc::Sender<TakerRequest>) -> Self {
        let taker = Self { tx };
        taker.send_taker_offer().await.unwrap();
        taker
    }

    async fn send_taker_offer(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = TakerRequest::SendTakerOffer { rsp_tx };
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

pub(super) enum TakerRequest {
    SendTakerOffer {
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    },
    RegisterTradeNotifTx,
    UnregisterTradeNotifTx,
}

struct TakerActor {
    rx: mpsc::Receiver<TakerRequest>,
    interfacer_handle: InterfacerHandle,
    order_envelope: OrderEnvelope,
    offer: Offer,
    offer_event_id: Option<EventIdString>,
    trade_rsp_envelope: Option<TradeResponseEnvelope>,
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
        }
    }

    async fn run(&mut self) {
        loop {
            select! {
                Some(request) = self.rx.recv() => {
                    self.handle_request(request).await;
                },
                else => break,

            }
        }
    }

    async fn handle_request(&mut self, request: TakerRequest) {
        match request {
            TakerRequest::SendTakerOffer { rsp_tx } => self.send_taker_offer(rsp_tx).await,
            TakerRequest::RegisterTradeNotifTx => todo!(),
            TakerRequest::UnregisterTradeNotifTx => todo!(),
        }
    }

    async fn send_taker_offer(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order_envelope = self.order_envelope.clone();
        let offer = self.offer.clone();

        let result = self
            .interfacer_handle
            .send_taker_offer_message(
                order_envelope.pubkey,
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
}
