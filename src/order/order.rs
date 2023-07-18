use super::{obligation::*, trade_details::*, trade_engine_details::*};
use serde::Serialize;

pub struct Order<T: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    pub trade_uuid: String, // TODO: Change to UUID type
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub engine_details: TradeEngineDetails<T>,
    pub pow_difficulty: u64,
}
