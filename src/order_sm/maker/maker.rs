use std::collections::HashMap;

use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use uuid::Uuid;

use crate::{
    common::{
        error::N3xbError,
        types::{SerdeGenericTrait, SerdeGenericType},
    },
    interfacer::InterfacerHandle,
    offer::Offer,
    order::Order,
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

    async fn make_new_order(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::SendMakerOrder { rsp_tx };
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
    QueryOffers,
    SendTradeResponse,
    RegisterNotifTx,
    UnregisterNotifTx,
}

struct MakerActor {
    rx: mpsc::Receiver<MakerRequest>,
    interfacer_handle: InterfacerHandle,
    order: Order,
    offers: HashMap<Uuid, Offer>,
}

impl MakerActor {
    pub(crate) async fn new(
        rx: mpsc::Receiver<MakerRequest>,
        interfacer_handle: InterfacerHandle,
        order: Order,
    ) -> Self {
        let offers: HashMap<Uuid, Offer> = HashMap::new();

        MakerActor {
            rx,
            interfacer_handle,
            order,
            offers,
        }
    }

    async fn run(&mut self) {
        let (tx, mut rx) = mpsc::channel::<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>(20);

        self.interfacer_handle
            .register_peer_message_tx(self.order.trade_uuid.clone(), tx)
            .await
            .unwrap();

        loop {
            select! {
                Some(request) = self.rx.recv() => {
                    self.handle_request(request).await;
                },
                Some((peer_message_type, peer_message)) = rx.recv() => {
                    self.handle_peer_message(peer_message_type, peer_message).await;
                },
                else => break,
            }
        }
    }

    async fn handle_request(&mut self, request: MakerRequest) {
        match request {
            MakerRequest::SendMakerOrder { rsp_tx } => self.send_maker_order(rsp_tx).await,
            MakerRequest::QueryOffers => todo!(),
            MakerRequest::SendTradeResponse => todo!(),
            MakerRequest::RegisterNotifTx => todo!(),
            MakerRequest::UnregisterNotifTx => todo!(),
        }
    }

    async fn handle_peer_message(
        &mut self,
        peer_message_type: SerdeGenericType,
        peer_message: Box<dyn SerdeGenericTrait>,
    ) {
        match peer_message_type {
            SerdeGenericType::TakerOffer => {
                let offer = peer_message.downcast_ref::<Offer>().expect("Received peer message of SerdeGenericType::TakerOffer, but failed to downcast into message into Offer");
                self.handle_taker_offer(offer);
            }

            SerdeGenericType::TradeResponse => {
                todo!();
            }

            SerdeGenericType::TradeEngineSpecific => {
                todo!();
            }
        }
    }

    fn handle_taker_offer(&mut self, offer: &Offer) {
        // TODO:
        // Confirm that the offer is valid and compatible with the initial order
        match offer.validate_against(&self.order) {
            Ok(_) => {
                // For offers, add to lists of offers

                // Notify user of new offer recieved
            }

            Err(_error) => {
                // Reject offer by sending Taker a Trade Response message

                // Notify user that invalid offer was received
            }
        }
    }

    async fn send_maker_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order = self.order.clone();
        let result = self.interfacer_handle.send_maker_order_note(order).await;
        rsp_tx.send(result).unwrap(); // oneshot should not fail
    }
}
