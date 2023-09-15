use std::sync::{Arc, Mutex};

use crate::{common::error::N3xbError, interfacer::InterfacerHandle, offer::Offer, order::Order};

pub type ArcTakerSM = Arc<Mutex<TakerSM>>;

pub struct TakerSM {
    interfacer_handle: InterfacerHandle,
    order: Order,
    offer: Offer,
}

impl TakerSM {
    pub(crate) async fn new(
        interfacer_handle: InterfacerHandle,
        order: Order,
        offer: Offer,
    ) -> Result<TakerSM, N3xbError> {
        interfacer_handle
            .send_taker_offer_message(
                order.pubkey.clone(),
                order.event_id.clone(),
                order.trade_uuid.clone(),
                offer.clone(),
            )
            .await?;

        let taker_sm = TakerSM {
            interfacer_handle,
            order,
            offer,
        };

        Ok(taker_sm)
    }
}
