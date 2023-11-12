use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::{
    common::types::SerdeGenericTrait,
    trade_rsp::{TradeResponse, TradeResponseBuilder, TradeResponseStatus},
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

    pub fn check(trade_rsp: &TradeResponse, expected: &TradeResponse) {
        assert_eq!(trade_rsp.trade_response, expected.trade_response);
        for i in 0..trade_rsp.reject_reason.len() {
            assert_eq!(trade_rsp.reject_reason[i], expected.reject_reason[i]);
        }
        let test_trade_engine_specifics = trade_rsp
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineTradeRspSpecifics>()
            .unwrap();
        let expected_test_trade_engine_specifics = expected
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineTradeRspSpecifics>()
            .unwrap();
        assert_eq!(
            test_trade_engine_specifics.test_specific_field,
            expected_test_trade_engine_specifics.test_specific_field
        );
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
