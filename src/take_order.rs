use erased_serde::Serialize as ErasedSerialize;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use typetag;

use crate::peer_messaging;

// Take Order Message Data Structure

#[derive(Debug, Deserialize, Serialize)]
pub struct Obligation {
  amount: u64,
  currency: String,  // TODO: Change to ISO 4217 Enum
  payment: String,  // TODO: Change to Payment Method Enum
  bond_amount: Option<u64>,
}

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + Debug {}

#[derive(Debug, Deserialize, Serialize)]
pub struct TakeOrderMessage {
  maker_obligation: Obligation,
  taker_obligation: Obligation,
  market_oracle_used: Option<String>,  // TODO: Change to URL type
  trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
  pow_difficulty: u64,
}

#[typetag::serde(name = "n3xB-take-order")]
impl peer_messaging::PeerMessageTrait for TakeOrderMessage {}