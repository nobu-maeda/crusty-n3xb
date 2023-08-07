use super::peer_messaging::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Trade Response Message Data Structure
pub trait SerdeGenericTrait: Debug {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeResponseMessage<T: SerdeGenericTrait> {
    trade_response: String,             // TODO: Change to Enums
    reject_reason: Option<Vec<String>>, // TODO: Change to Enums
    trade_engine_specifics: T,
}
// TODO: What about deserialization?
impl<T: SerdeGenericTrait> PeerMessageTrait for TradeResponseMessage<T> {}
