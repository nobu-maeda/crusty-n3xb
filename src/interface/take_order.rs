use super::peer_messaging::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typetag;

// Take Order Message Data Structure

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Obligation {
    amount: u64,
    currency: String, // TODO: Change to ISO 4217 Enum
    payment: String,  // TODO: Change to Payment Method Enum
    bond_amount: Option<u64>,
}

#[typetag::serialize(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: Debug {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TakeOrderMessage<T: TradeEngineSpecfiicsTrait + Serialize + Clone> {
    maker_obligation: Obligation,
    taker_obligation: Obligation,
    market_oracle_used: Option<String>, // TODO: Change to URL type
    trade_engine_specifics: T,
    pow_difficulty: u64,
}

#[typetag::serialize(name = "n3xB-take-order")] // TODO: What about deserialization?
impl<T: TradeEngineSpecfiicsTrait + Serialize + Clone> PeerMessageTrait for TakeOrderMessage<T> {}
