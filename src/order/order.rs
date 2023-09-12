use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{obligation::*, trade_details::*};
use crate::common::types::SerdeGenericTrait;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub pubkey: String,
    pub event_id: String,
    pub trade_uuid: String, // TODO: Change to UUID type
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub trade_engine_specifics: Arc<dyn SerdeGenericTrait>,
    pub pow_difficulty: u64,
}
