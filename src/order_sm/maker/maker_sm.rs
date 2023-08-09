use std::sync::{Arc, Mutex};

use crate::{
    common::error::N3xbError, common::types::SerdeGenericTrait, interface::ArcInterface,
    order::Order,
};

pub type ArcMakerSM<T> = Arc<Mutex<MakerSM<T>>>;

#[derive(Clone)]
pub struct MakerSM<EngineSpecificsType: SerdeGenericTrait> {
    interface: ArcInterface<EngineSpecificsType>,
    order: Order<EngineSpecificsType>,
}

impl<EngineSpecificsType: SerdeGenericTrait> MakerSM<EngineSpecificsType> {
    pub async fn new(
        interface: ArcInterface<EngineSpecificsType>,
        order: Order<EngineSpecificsType>,
    ) -> Result<MakerSM<EngineSpecificsType>, N3xbError> {
        let maker_sm = MakerSM {
            interface: Arc::clone(&interface),
            order: order.clone(),
        };
        interface
            .lock()
            .unwrap()
            .send_maker_order_note(order)
            .await?;
        Ok(maker_sm)
    }
}
