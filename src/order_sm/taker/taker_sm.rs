use std::sync::{Arc, Mutex};

use crate::{
    common::{error::N3xbError, types::SerdeGenericTrait},
    interface::ArcInterface,
    offer::Offer,
    order::Order,
};

pub type ArcTakerSM<T, U> = Arc<Mutex<TakerSM<T, U>>>;

#[derive(Clone)]
pub struct TakerSM<
    OrderEngineSpecificType: SerdeGenericTrait,
    OfferEngineSpecificType: SerdeGenericTrait,
> {
    interface: ArcInterface<OrderEngineSpecificType, OfferEngineSpecificType>,
    order: Order<OrderEngineSpecificType>,
    offer: Offer<OfferEngineSpecificType>,
}

impl<OrderEngineSpecificType: SerdeGenericTrait, OfferEngineSpecificType: SerdeGenericTrait>
    TakerSM<OrderEngineSpecificType, OfferEngineSpecificType>
{
    pub async fn new(
        interface: ArcInterface<OrderEngineSpecificType, OfferEngineSpecificType>,
        order: Order<OrderEngineSpecificType>,
        offer: Offer<OfferEngineSpecificType>,
    ) -> Result<TakerSM<OrderEngineSpecificType, OfferEngineSpecificType>, N3xbError> {
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
