use std::sync::{Arc, Mutex};

use crate::{common::error::N3xbError, interface::ArcInterface, order::Order};

pub type ArcMakerSM = Arc<Mutex<MakerSM>>;

pub struct MakerSM {
    interface: ArcInterface,
    order: Order,
    // There's no explicit state variable being tracked for now
    // States are instead determined by the following
    //
    //
}

impl MakerSM {
    pub async fn new(interface: ArcInterface, order: Order) -> Result<MakerSM, N3xbError> {
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
