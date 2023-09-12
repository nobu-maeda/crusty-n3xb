use std::sync::{Arc, Mutex};

use crate::{common::error::N3xbError, interface::ArcInterface, offer::Offer, order::Order};

pub type ArcTakerSM = Arc<Mutex<TakerSM>>;

pub struct TakerSM {
    interface: ArcInterface,
    order: Order,
    offer: Offer,
}

impl TakerSM {
    pub async fn new(
        interface: ArcInterface,
        order: Order,
        offer: Offer,
    ) -> Result<TakerSM, N3xbError> {
        let taker_sm = TakerSM {
            interface: Arc::clone(&interface),
            order: order.clone(),
            offer: offer.clone(),
        };

        interface
            .lock()
            .unwrap()
            .send_taker_offer_message(order.pubkey, order.event_id, order.trade_uuid, offer)
            .await?;
        Ok(taker_sm)
    }
}
