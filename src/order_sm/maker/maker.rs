use std::{borrow::Borrow, collections::HashMap};

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
        match Self::validate_offer(offer, &self.order) {
            Ok(_) => {
                // For offers, add to lists of offers

                // Notify user of new offer recieved
            }

            Err(error) => {
                // Reject offer by sending Taker a Trade Response message

                // Notify user that invalid offer was received
            }
        }
    }

    fn validate_offer(offer: &Offer, order: &Order) -> Result<(), N3xbError> {
        Self::validate_maker_obligation(offer, order)?;
        Self::validate_taker_obligation(offer, order)?;

        // Check Taker suggested PoW difficulty is higher than in initial Maker Order
        if let Some(pow_difficulty) = offer.pow_difficulty {
            if pow_difficulty < order.pow_difficulty {
                return Err(N3xbError::Simple(format!(
                    "Taker Offer suggested lower PoW difficulty than specified in initial Order"
                )));
            }
        }

        // TODO: How to validate trade engine specifics? Depend on the Trade Engine to do so after it gets notified?

        Ok(())
    }

    fn f64_amount_within_pct_of(float1: f64, float2: f64, pct: f64) -> bool {
        let max = float1 * (1.0 + pct / 100.0);
        let min = float1 * (1.0 - pct / 100.0);
        return min <= float2 && float2 >= max;
    }

    fn validate_maker_obligation(offer: &Offer, order: &Order) -> Result<(), N3xbError> {
        if !order
            .maker_obligation
            .kinds
            .contains(&offer.maker_obligation.kind)
        {
            return Err(N3xbError::Simple(format!(
                "Offer Maker Obligation Kind {} not found in initial Order",
                offer.maker_obligation.kind
            )));
        }

        if let Some(amount_min) = order.maker_obligation.content.amount_min {
            if offer.maker_obligation.amount < amount_min
                || offer.maker_obligation.amount > order.maker_obligation.content.amount
            {
                return Err(N3xbError::Simple(format!(
                    "Offer Maker Obligation amount not within bounds specificed in initial Order"
                )));
            }
        } else if offer.maker_obligation.amount != order.maker_obligation.content.amount {
            return Err(N3xbError::Simple(format!(
                "Offer Maker Obligation amount does not match amount specified in initial Order"
            )));
        }

        if let Some(maker_bond_pct) = order.trade_details.content.maker_bond_pct {
            let order_bond_amount =
                maker_bond_pct as f64 / 100.0 * offer.maker_obligation.amount as f64;

            // Should be okay to give +/- 0.1% leeway for bond amount
            if let Some(offer_bond_amount) = offer.maker_obligation.bond_amount {
                if !Self::f64_amount_within_pct_of(order_bond_amount, offer_bond_amount as f64, 0.1)
                {
                    return Err(N3xbError::Simple(format!("Offer Maker Obligation bond amount does not match percentage specified in initial Order")));
                }
            } else {
                return Err(N3xbError::Simple(format!("Offer Maker Obligation does not have bond amount specified as required in the initial Order")));
            }
        }

        Ok(())
    }

    fn validate_taker_obligation(offer: &Offer, order: &Order) -> Result<(), N3xbError> {
        if !order
            .taker_obligation
            .kinds
            .contains(&offer.taker_obligation.kind)
        {
            return Err(N3xbError::Simple(format!(
                "Offer Taker Obligation Kind {} not found in initial Order",
                offer.taker_obligation.kind
            )));
        }

        let maker_amount = offer.maker_obligation.amount as f64; // This is validated in Maker validation. So we take it as it is

        if let Some(limit_rate) = order.taker_obligation.content.limit_rate {
            let expected_taker_amount = maker_amount * limit_rate;
            if !Self::f64_amount_within_pct_of(
                expected_taker_amount,
                offer.taker_obligation.amount as f64,
                0.1,
            ) {
                return Err(N3xbError::Simple(format!(
                    "Offer Taker Obligation amount not as expected"
                )));
            }
        }

        if let Some(market_oracle_used) = &offer.market_oracle_used {
            if let Some(market_oracles) = &order.taker_obligation.content.market_oracles {
                if !market_oracles.contains(market_oracle_used) {
                    return Err(N3xbError::Simple(format!(
                        "Market Oracle {} not found in list of the initial Order",
                        market_oracle_used
                    )));
                }
            } else {
                return Err(N3xbError::Simple(format!(
                        "Market Oracle {} not expected when intiial Order contains no allowable oracles",
                        market_oracle_used
                    )));
            }
        }

        if order.taker_obligation.content.market_offset_pct.is_some() {
            return Err(N3xbError::Simple(format!(
                "Market & Oracle based rate determination not yet supported "
            )));
        }

        if let Some(taker_bond_pct) = order.trade_details.content.taker_bond_pct {
            let order_bond_amount =
                taker_bond_pct as f64 / 100.0 * offer.taker_obligation.amount as f64;

            // Should be okay to give +/- 0.1% leeway for bond amount
            if let Some(offer_bond_amount) = offer.taker_obligation.bond_amount {
                if !Self::f64_amount_within_pct_of(order_bond_amount, offer_bond_amount as f64, 0.1)
                {
                    return Err(N3xbError::Simple(format!("Offer Taker Obligation bond amount does not match percentage specified in initial Order")));
                }
            } else {
                return Err(N3xbError::Simple(format!("Offer Taker Obligation does not have bond amount specified as required in the initial Order")));
            }
        }

        Ok(())
    }

    async fn send_maker_order(&mut self, rsp_tx: oneshot::Sender<Result<(), N3xbError>>) {
        let order = self.order.clone();
        let result = self.interfacer_handle.send_maker_order_note(order).await;
        rsp_tx.send(result).unwrap(); // oneshot should not fail
    }
}
