use serde::{Deserialize, Serialize};

use std::{collections::HashSet, fmt::Debug};

use crate::common::types::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MakerObligation {
    pub kinds: HashSet<ObligationKind>,
    pub content: MakerObligationContent,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TakerObligation {
    pub kinds: HashSet<ObligationKind>,
    pub content: TakerObligationContent,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct MakerObligationContent {
    pub amount: f64,
    pub amount_min: Option<f64>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TakerObligationContent {
    pub limit_rate: Option<f64>,
    pub market_offset_pct: Option<f64>,
    pub market_oracles: Option<HashSet<String>>, // TODO: Change to hashset of URL type
}
