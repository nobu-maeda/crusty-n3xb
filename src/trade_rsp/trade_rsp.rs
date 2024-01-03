use std::{any::Any, collections::HashSet, fmt::Debug};

use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::common::{error::OfferInvalidReason, types::*};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeResponseEnvelope {
    pub pubkey: XOnlyPublicKey,
    pub urls: HashSet<Url>,
    pub event_id: EventIdString,
    pub trade_rsp: TradeResponse,
    pub(crate) _private: (),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum TradeResponseStatus {
    Accepted,
    Rejected,
    NotAvailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeResponse {
    pub offer_event_id: EventIdString,
    pub trade_response: TradeResponseStatus,
    pub reject_reason: Vec<OfferInvalidReason>,
    pub trade_engine_specifics: Box<dyn SerdeGenericTrait>,
}

#[typetag::serde(name = "n3xB-trade-response")]
impl SerdeGenericTrait for TradeResponse {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
