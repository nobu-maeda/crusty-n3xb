use crate::{common::error::N3xbError, interfacer::InterfacerHandle, offer::Offer, order::Order};

pub struct TakerSM {
    interfacer_handle: InterfacerHandle,
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
                offer,
            )
            .await?;

        let taker_sm = TakerSM { interfacer_handle };

        Ok(taker_sm)
    }
}
