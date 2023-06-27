use serde::{Serialize, Deserialize};

use std::collections::HashSet;
use std::fmt::*;

#[derive(Clone, Debug)]
pub struct TradeDetails {
  pub parameters: HashSet<TradeParameter>,
  pub content: TradeDetailsContent,
}

impl TradeDetails {
  pub fn parameters_to_tags(&self) -> Vec<String> {
    let mut tag_string_vec: Vec<String> = vec![];
    for parameter in self.parameters.iter() {
      tag_string_vec.push(parameter.to_tag());
    }
    tag_string_vec
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeDetailsContent {
  pub maker_bond_pct: Option<u32>,
  pub taker_bond_pct: Option<u32>,
  pub trade_timeout: Option<u32>,
}

#[derive(Clone, Debug)]
pub enum TradeParameter {
  MakerHasReputation,
  TakerReputationRequired,
  BondsRequired,
  TrustedEscrow,
  TrustlessEscrow,
  TrustedArbitration,
  AcceptsPartialTake,
  TradeTimesOut(TradeTimeOutLimit)
}

impl TradeParameter {
  pub fn to_tag(&self) -> String {
    match self {
      TradeParameter::MakerHasReputation => "maker-has-reputation".to_string(),
      TradeParameter::TakerReputationRequired => "taker-reputation-required".to_string(),
      TradeParameter::BondsRequired => "bonds-required".to_string(),
      TradeParameter::TrustedEscrow => "trusted-escrow".to_string(),
      TradeParameter::TrustlessEscrow => "trustless-escrow".to_string(),
      TradeParameter::TrustedArbitration => "trusted-arbitration".to_string(),
      TradeParameter::AcceptsPartialTake => "accepts-partial-take".to_string(),
      TradeParameter::TradeTimesOut(timelimit) => format!("trade-times-out{}", timelimit.to_tag()),
    }
  }
}

impl Display for TradeParameter {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      TradeParameter::MakerHasReputation => write!(f, "Maker has reputation"),
      TradeParameter::TakerReputationRequired => write!(f, "Taker reputation required"),
      TradeParameter::BondsRequired => write!(f, "Bonds required"),
      TradeParameter::TrustedEscrow => write!(f, "Trusted escrow"),
      TradeParameter::TrustlessEscrow => write!(f, "Trustless escrow"),
      TradeParameter::TrustedArbitration => write!(f, "Trusted arbitration"),
      TradeParameter::AcceptsPartialTake => write!(f, "Accepts partial take"),
      TradeParameter::TradeTimesOut(timelimit) => write!(f, "Trade Times out with {:?} limit", timelimit),
    }
  }
}

#[derive(Clone, Debug)]
pub enum TradeTimeOutLimit {
  FourDays,
  OneDay,
  NoTimeout,
  TradeEngineSpecific,
}

impl TradeTimeOutLimit {
  pub fn to_tag(&self) -> String {
    match self {
      TradeTimeOutLimit::FourDays => "-4-days".to_string(),
      TradeTimeOutLimit::OneDay => "-24-hours".to_string(),
      TradeTimeOutLimit::NoTimeout => "-none".to_string(),
      TradeTimeOutLimit::TradeEngineSpecific => "".to_string(),
    }
  }
}

impl Display for TradeTimeOutLimit {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      TradeTimeOutLimit::FourDays => write!(f, "4 days"),
      TradeTimeOutLimit::OneDay => write!(f, "1 day"),
      TradeTimeOutLimit::NoTimeout => write!(f, "no timeout"),
      TradeTimeOutLimit::TradeEngineSpecific => write!(f, "trade engine specific"),
    }
  }
}