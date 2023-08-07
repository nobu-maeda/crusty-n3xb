use crate::{common::SerdeGenericTrait, error::N3xbError, interface::ArcInterface, order::Order};

pub struct TakerSM<'a, EngineSpecificsType: SerdeGenericTrait> {
    interface: &'a ArcInterface<EngineSpecificsType>,
    order: Order<EngineSpecificsType>,
}

impl<'a, EngineSpecificsType: SerdeGenericTrait> TakerSM<'a, EngineSpecificsType> {
    pub async fn new(
        interface: &'a ArcInterface<EngineSpecificsType>,
        order: Order<EngineSpecificsType>,
    ) -> Result<TakerSM<'a, EngineSpecificsType>, N3xbError> {
        let taker_sm: TakerSM<'_, EngineSpecificsType> = TakerSM {
            interface,
            order: order.clone(),
        };
        // interface
        //     .lock()
        //     .unwrap()
        //     .send_take_order_message(order)
        //     .await?;
        Ok(taker_sm)
    }
}
