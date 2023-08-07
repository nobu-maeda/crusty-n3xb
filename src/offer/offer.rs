use crate::common::SerdeGenericTrait;
use create::interface::peer_messaging::*;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Take Order Message Data Structure

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Obligation {
    amount: u64,
    currency: String, // TODO: Change to ISO 4217 Enum
    payment: String,  // TODO: Change to Payment Method Enum
    bond_amount: Option<u64>,
}

pub trait SerdeGenericTrait: Debug {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TakeOrderMessage<T: SerdeGenericTrait> {
    maker_obligation: Obligation,
    taker_obligation: Obligation,
    market_oracle_used: Option<String>, // TODO: Change to URL type
    trade_engine_specifics: T,
    pow_difficulty: u64,
}
// TODO: What about deserialization?
impl<T: SerdeGenericTrait> PeerMessageTrait for TakeOrderMessage<T> {}
