use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::{
    common::types::SerdeGenericTrait,
    trade_rsp::{TradeResponseBuilder, TradeResponseStatus},
};

use super::SomeTestParams;

pub struct SomeTestTradeRspParams {}

impl SomeTestTradeRspParams {
    pub fn default_builder() -> TradeResponseBuilder {
        let mut builder: TradeResponseBuilder = TradeResponseBuilder::new();
        builder.trade_response(TradeResponseStatus::Accepted);

        let trade_engine_specifics = Box::new(SomeTradeEngineTradeRspSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);
        builder
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SomeTradeEngineTradeRspSpecifics {
    pub test_specific_field: String,
}

#[typetag::serde(name = "some-trade-engine-trade-rsp-specifics")]
impl SerdeGenericTrait for SomeTradeEngineTradeRspSpecifics {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
