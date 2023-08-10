use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::fmt::Debug;

use crate::common::types::*;

// Take Order Message Data Structure

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Obligation {
    kind: ObligationKind,
    amount: u64,
    bond_amount: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Offer<T: SerdeGenericTrait> {
    maker_obligation: Obligation,
    taker_obligation: Obligation,
    market_oracle_used: Option<String>, // TODO: Change to URL type
    trade_engine_specifics: T,
    pow_difficulty: u64,
}

impl<T: SerdeGenericTrait> SerdeGenericTrait for Offer<T> {}
