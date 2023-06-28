use dyn_clone::DynClone;
use erased_serde::Serialize as ErasedSerialize;

use std::fmt::*;

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + DynClone + Debug {}

dyn_clone::clone_trait_object!(TradeEngineSpecfiicsTrait);

#[derive(Clone, Debug)]
pub struct TradeEngineDetails {
    pub trade_engine_name: String,
    pub trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
}
