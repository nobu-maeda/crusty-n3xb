use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{obligation::*, trade_details::*};
use crate::common::types::SerdeGenericTrait;

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub pubkey: XOnlyPublicKey,
    pub event_id: String,
    pub trade_uuid: Uuid, // TODO: Change to UUID type
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    pub pow_difficulty: u64,
}
