use std::sync::{Arc, Mutex};

use crate::{common::error::N3xbError, interfacer::InterfacerHandle, order::Order};

pub type ArcMakerSM = Arc<Mutex<MakerSM>>;

pub struct MakerSM {
    interfacer_handle: InterfacerHandle,
    order: Order,
    // There's no explicit state variable being tracked for now
    // States are instead determined by the following
    //
    //
}

impl MakerSM {
    pub(crate) async fn new(
        interfacer_handle: InterfacerHandle,
        order: Order,
    ) -> Result<MakerSM, N3xbError> {
        // TODO: Subscribe to any inbound peer messages regarding Order this MakerSM tracks

        interfacer_handle
            .send_maker_order_note(order.clone())
            .await?;

        let maker_sm = MakerSM {
            interfacer_handle,
            order: order,
        };
        Ok(maker_sm)
    }

    // TODO: Add ability for Trade Engine to subscribe to state update of the MakerSM

    // TODO: Function for Trade Engine to query all offers

    // TODO: Function for Trade Engine to accept or reject an offer
}
