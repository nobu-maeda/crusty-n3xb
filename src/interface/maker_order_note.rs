use crate::common::types::SerdeGenericTrait;
use crate::order::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MakerOrderNote {
    pub maker_obligation: MakerObligationContent,
    pub taker_obligation: TakerObligationContent,
    pub trade_details: TradeDetailsContent,
    pub trade_engine_specifics: Arc<dyn SerdeGenericTrait>,
    pub pow_difficulty: u64,
}
