use std::sync::{Arc, Mutex};

use crate::{
    common::error::N3xbError, common::types::SerdeGenericTrait, interface::ArcInterface,
    order::Order,
};

pub type ArcMakerSM<T, U> = Arc<Mutex<MakerSM<T, U>>>;

#[derive(Clone)]
pub struct MakerSM<
    OrderEngineSpecificType: SerdeGenericTrait,
    OfferEngineSpecificType: SerdeGenericTrait,
> {
    interface: ArcInterface<OrderEngineSpecificType, OfferEngineSpecificType>,
    order: Order<OrderEngineSpecificType>,
    // There's no explicit state variable being tracked for now
    // States are instead determined by the following
    //
    //
}

impl<OrderEngineSpecificType: SerdeGenericTrait, OfferEngineSpecificType: SerdeGenericTrait>
    MakerSM<OrderEngineSpecificType, OfferEngineSpecificType>
{
    pub async fn new(
        interface: ArcInterface<OrderEngineSpecificType, OfferEngineSpecificType>,
        order: Order<OrderEngineSpecificType>,
    ) -> Result<MakerSM<OrderEngineSpecificType, OfferEngineSpecificType>, N3xbError> {
        let maker_sm = MakerSM {
            interface: Arc::clone(&interface),
            order: order.clone(),
        };

        // TODO: Subscribe to any inbound peer messages regarding Order this MakerSM tracks

        interface
            .lock()
            .unwrap()
            .send_maker_order_note(order)
            .await?;
        Ok(maker_sm)
    }

    // TODO: Add ability for Trade Engine to subscribe to state update of the MakerSM

    // TODO: Function for Trade Engine to query all offers

    // TODO: Function for Trade Engine to accept or reject an offer
}
