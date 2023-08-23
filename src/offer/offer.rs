use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::fmt::Debug;

use crate::common::types::*;

// Take Order Message Data Structure

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Obligation {
    pub kind: ObligationKind,
    pub amount: u64,
    pub bond_amount: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Offer<T: SerdeGenericTrait> {
    pub maker_obligation: Obligation,
    pub taker_obligation: Obligation,
    pub market_oracle_used: Option<String>, // TODO: Change to URL type
    pub trade_engine_specifics: T,
    pub pow_difficulty: Option<u64>,
}

impl<T: SerdeGenericTrait> SerdeGenericTrait for Offer<T> {}
