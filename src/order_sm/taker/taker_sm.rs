use crate::{
    error::N3xbError,
    interface::ArcInterface,
    order::{Order, TradeEngineSpecfiicsTrait},
};

pub struct TakerSM<'a, EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone> {
    interface: &'a ArcInterface<EngineSpecificsType>,
    order: Order<EngineSpecificsType>,
}

impl<'a, EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone> TakerSM<'a, EngineSpecificsType> {
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
