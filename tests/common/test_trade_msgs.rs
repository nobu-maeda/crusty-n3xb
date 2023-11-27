use std::any::Any;

use serde::{Deserialize, Serialize};

use crusty_n3xb::common::types::SerdeGenericTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SomeTradeEngMsg {
    pub(crate) some_trade_specific_field: String,
}

#[typetag::serde(name = "some-trade-eng-msg")]
impl SerdeGenericTrait for SomeTradeEngMsg {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl SomeTradeEngMsg {
    pub(crate) fn some_trade_specific_string() -> String {
        "SomeTradeSpecificString".to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct AnotherTradeEngMsg {
    pub(crate) another_trade_specific_field: String,
}

#[typetag::serde(name = "another-trade-eng-msg")]
impl SerdeGenericTrait for AnotherTradeEngMsg {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl AnotherTradeEngMsg {
    pub(crate) fn another_trade_specific_string() -> String {
        "AnotherTradeSpecificString".to_string()
    }
}
