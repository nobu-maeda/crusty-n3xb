use serde::{Deserialize, Serialize};

use super::{obligation::*, trade_details::*, trade_engine_details::*};

#[derive(Debug, Deserialize, Serialize)]
pub struct MakerOrderNote {
    pub maker_obligation: MakerObligationContent,
    pub taker_obligation: TakerObligationContent,
    pub trade_details: TradeDetailsContent,
    pub trade_engine_specifics: Option<Box<dyn TradeEngineSpecfiicsTrait>>,
    pub pow_difficulty: u64,
}
