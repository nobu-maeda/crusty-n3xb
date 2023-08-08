use crate::{
    common::types::SerdeGenericTrait, error::N3xbError, interface::ArcInterface, order::Order,
};

pub struct MakerSM<'a, EngineSpecificsType: SerdeGenericTrait> {
    interface: &'a ArcInterface<EngineSpecificsType>,
    order: Order<EngineSpecificsType>,
}

impl<'a, EngineSpecificsType: SerdeGenericTrait> MakerSM<'a, EngineSpecificsType> {
    pub async fn new(
        interface: &'a ArcInterface<EngineSpecificsType>,
        order: Order<EngineSpecificsType>,
    ) -> Result<MakerSM<'a, EngineSpecificsType>, N3xbError> {
        let maker_sm = MakerSM {
            interface,
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
