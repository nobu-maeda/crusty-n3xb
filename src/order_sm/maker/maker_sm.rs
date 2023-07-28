use crate::{
    interface::ArcInterface,
    order::{Order, TradeEngineSpecfiicsTrait},
};

pub struct MakerSM<'a, EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone> {
    interface: &'a ArcInterface<EngineSpecificsType>,
    order: Order<EngineSpecificsType>,
}

impl<'a, EngineSpecificsType: TradeEngineSpecfiicsTrait + Clone> MakerSM<'a, EngineSpecificsType> {
    pub async fn new(
        interface: &'a ArcInterface<EngineSpecificsType>,
        order: Order<EngineSpecificsType>,
    ) -> MakerSM<'a, EngineSpecificsType> {
        let maker_sm = MakerSM {
            interface,
            order: order.clone(),
        };
        interface.lock().unwrap().send_maker_order_note(order).await;
        maker_sm
    }
}
