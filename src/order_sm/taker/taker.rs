use tokio::{
    select,
    sync::{mpsc, oneshot},
};

use crate::{common::error::N3xbError, interfacer::InterfacerHandle, offer::Offer, order::Order};

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
        order: Order,
        offer: Offer,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<TakerRequest>(Self::TAKER_REQUEST_CHANNEL_SIZE);
        let mut actor = TakerActor::new(rx, interfacer_handle, order, offer).await;
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
    RegisterNotifTx,
    UnregisterNotifTx,
}

struct TakerActor {
    rx: mpsc::Receiver<TakerRequest>,
    interfacer_handle: InterfacerHandle,
    order: Order,
    offer: Offer,
}

impl TakerActor {
    pub(crate) async fn new(
        rx: mpsc::Receiver<TakerRequest>,
        interfacer_handle: InterfacerHandle,
        order: Order,
        offer: Offer,
    ) -> Self {
        TakerActor {
            rx,
            interfacer_handle,
            order,
            offer,
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
            TakerRequest::RegisterNotifTx => todo!(),
            TakerRequest::UnregisterNotifTx => todo!(),
        }
    }

    async fn send_taker_offer(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let offer = self.offer.clone();

        let result = self
            .interfacer_handle
            .send_taker_offer_message(
                self.order.pubkey.clone(),
                self.order.event_id.clone(),
                self.order.trade_uuid.clone(),
                offer,
            )
            .await;
        rsp_tx.send(result).unwrap(); // oneshot should not fail
    }
}
