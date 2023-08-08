use crate::common::types::*;
use iso_currency::Currency;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Take Order Message Data Structure

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ObligationKind {
    Bitcoin(BitcoinSettlementMethod),
    Fiat(Currency, FiatPaymentMethod),
    Custom(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Obligation {
    kind: ObligationKind,
    amount: u64,
    bond_amount: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct TakeOrderMessage<T: SerdeGenericTrait> {
    maker_obligation: Obligation,
    taker_obligation: Obligation,
    market_oracle_used: Option<String>, // TODO: Change to URL type
    trade_engine_specifics: T,
    pow_difficulty: u64,
}
