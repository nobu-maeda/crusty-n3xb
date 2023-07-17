use super::{obligation::*, trade_details::*, trade_engine_details::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MakerOrderNote<T: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    pub maker_obligation: MakerObligationContent,
    pub taker_obligation: TakerObligationContent,
    pub trade_details: TradeDetailsContent,
    pub trade_engine_specifics: T,
    pub pow_difficulty: u64,
}
