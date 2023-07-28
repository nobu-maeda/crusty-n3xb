use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait TradeEngineSpecfiicsTrait: Serialize + DeserializeOwned + Clone + Debug {}
