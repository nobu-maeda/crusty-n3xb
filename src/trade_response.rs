use erased_serde::Serialize as ErasedSerialize;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typetag;

use crate::peer_messaging;

// Trade Response Message Data Structure

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + Debug {}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeResponseMessage {
    trade_response: String,             // TODO: Change to Enums
    reject_reason: Option<Vec<String>>, // TODO: Change to Enums
    trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
}

#[typetag::serde(name = "n3xB-trade-response")]
impl peer_messaging::PeerMessageTrait for TradeResponseMessage {}
