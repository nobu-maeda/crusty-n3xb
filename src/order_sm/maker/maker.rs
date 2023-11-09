use log::error;
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

    pub async fn make_new_order(&self) -> Result<(), N3xbError> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let request = MakerRequest::SendMakerOrder { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap()
    }

    pub async fn register_offer_notif_tx(
        &self,
        notif_tx: mpsc::Sender<Result<Offer, N3xbError>>,
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
    QueryOffers,
    SendTradeResponse,
    RegisterOfferNotifTx {
        tx: mpsc::Sender<Result<Offer, N3xbError>>,
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
    offers: HashMap<Uuid, Offer>,
    notif_tx: Option<mpsc::Sender<Result<Offer, N3xbError>>>,
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
            notif_tx: None,
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
            MakerRequest::RegisterOfferNotifTx { tx, rsp_tx } => {
                self.register_notif_tx(tx, rsp_tx).await;
            }
            MakerRequest::UnregisterOfferNotifTx { rsp_tx } => {
                self.unregister_notif_tx(rsp_tx).await;
            }
        }
    }

    async fn handle_peer_message(
        &mut self,
        peer_message_type: SerdeGenericType,
        peer_message: Box<dyn SerdeGenericTrait>,
    ) {
        match peer_message_type {
            SerdeGenericType::TakerOffer => {
                let offer = peer_message.downcast_ref::<Offer>().expect("Received peer message of SerdeGenericType::TakerOffer, but failed to downcast into message into Offer").to_owned();
                self.handle_taker_offer(offer).await;
            }

            SerdeGenericType::TradeResponse => {
                todo!();
            }

            SerdeGenericType::TradeEngineSpecific => {
                todo!();
            }
        }
    }

    async fn handle_taker_offer(&mut self, offer: Offer) {
        let valid = offer.validate_against(&self.order);
        match valid {
            Ok(_) => {
                self.offers.insert(offer.offer_uuid, offer.clone());

                // Notify user of new offer recieved
                if let Some(tx) = &self.notif_tx {
                    if let Some(error) = tx.send(Ok(offer)).await.err() {
                        error!(
                            "handle_taker_offer() failed in notifying user with mpsc - {}",
                            error
                        );
                    }
                }
            }

            Err(_error) => {
                // TODO: Reject offer by sending Taker a Trade Response message
            }
        }
    }

    async fn send_maker_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order = self.order.clone();
        let result = self.interfacer_handle.send_maker_order_note(order).await;
        rsp_tx.send(result).unwrap(); // oneshot should not fail
    }

    async fn register_notif_tx(
        &mut self,
        tx: mpsc::Sender<Result<Offer, N3xbError>>,
        rsp_tx: oneshot::Sender<Result<(), N3xbError>>,
    ) {
        let mut result = Ok(());
        if self.notif_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "Maker of Order {} already have notif_tx registered",
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
                "Maker of Order {} expected to already have notif_tx registered",
                self.order.trade_uuid
            ));
            result = Err(error);
        }
        self.notif_tx = None;
        rsp_tx.send(result).unwrap();
    }
}
