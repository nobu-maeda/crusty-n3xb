use super::{obligation::*, trade_details::*, trade_engine_details::*};

#[derive(Clone, Debug)]
pub struct Order<T: TradeEngineSpecfiicsTrait> {
    pub event_id: String,
    pub trade_uuid: String, // TODO: Change to UUID type
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub trade_engine_specifics: T,
    pub pow_difficulty: u64,
}
