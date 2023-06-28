use dyn_clone::DynClone;
use erased_serde::Serialize as ErasedSerialize;
use typetag;

use std::fmt::*;

#[derive(Clone, Debug)]
pub struct TradeEngineDetails {
    pub trade_engine_name: String,
    pub trade_engine_specifics: Option<Box<dyn TradeEngineSpecfiicsTrait>>,
}

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + DynClone + Debug {}

dyn_clone::clone_trait_object!(TradeEngineSpecfiicsTrait);
