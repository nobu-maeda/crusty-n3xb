use std::fmt::*;

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: Debug {}
