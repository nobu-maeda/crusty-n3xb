use serde::{Deserialize, Serialize};

use std::{any::Any, fmt::Debug, rc::Rc};

use crate::common::types::*;

// Take Order Message Data Structure

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obligation {
    pub kind: ObligationKind,
    pub amount: u64,
    pub bond_amount: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offer {
    pub maker_obligation: Obligation,
    pub taker_obligation: Obligation,
    pub market_oracle_used: Option<String>, // TODO: Change to URL type
    pub trade_engine_specifics: Rc<dyn SerdeGenericTrait>,
    pub pow_difficulty: Option<u64>,
}

#[typetag::serde(name = "n3xb_offer")]
impl SerdeGenericTrait for Offer {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
