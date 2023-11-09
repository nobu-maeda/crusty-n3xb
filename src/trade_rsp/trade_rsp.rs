use std::{any::Any, fmt::Debug};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::*;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum TradeResponseStatus {
    Accepted,
    Rejected,
    NotAvailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TradeRejectReason {
    Pending,
    InvalidMakerCurrency,
    InvalidMakerSettlemment,
    InvalidTakerCurrency,
    InvalidTakerSettlement,
    InvalidMarketOracle,
    MakerAmountOutOfRange,
    ExchangeRateOutOfRange,
    MakerBondOutOfRange,
    TakerBondOutOfRange,
    TradeEngineSpecific,
    PowTooHigh,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeResponse {
    pub offer_uuid: Uuid,
    pub trade_response: TradeResponseStatus,
    pub reject_reason: Vec<TradeRejectReason>,
    pub trade_engine_specifics: Box<dyn SerdeGenericTrait>,
}

#[typetag::serde(name = "n3xB-trade-response")]
impl SerdeGenericTrait for TradeResponse {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
