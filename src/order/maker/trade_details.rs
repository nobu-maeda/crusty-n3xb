use serde::{Serialize, Deserialize};

use std::collections::HashSet;
use std::fmt::*;

#[derive(Clone, Debug)]
pub struct TradeDetails {
  pub parameters: HashSet<TradeParameter>,
  pub content: TradeDetailsContent,
}

impl TradeDetails {
  pub fn parameters_to_tags(&self) -> HashSet<String> {
    let mut tag_string_set: HashSet<String> = HashSet::new();
    for parameter in self.parameters.iter() {
      tag_string_set.insert(parameter.to_tag());
      if let Some(parameter_prefix) = parameter.prefix() {
        tag_string_set.insert(parameter_prefix);
      }
    }
    tag_string_set
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeDetailsContent {
  pub maker_bond_pct: Option<u32>,
  pub taker_bond_pct: Option<u32>,
  pub trade_timeout: Option<u32>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
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
  fn to_tag(&self) -> String {
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

  fn prefix(&self) -> Option<String> {
    match self {
      TradeParameter::TradeTimesOut(timelimit) => {
        match timelimit {
          TradeTimeOutLimit::TradeEngineSpecific => None,
          TradeTimeOutLimit::FourDays => Some("trade-times-out".to_string()),
          TradeTimeOutLimit::OneDay => Some("trade-times-out".to_string()),
          TradeTimeOutLimit::NoTimeout => Some("trade-times-out".to_string()),
        }
      },
      TradeParameter::MakerHasReputation => None,
      TradeParameter::TakerReputationRequired => None,
      TradeParameter::BondsRequired => None,
      TradeParameter::TrustedEscrow => None,
      TradeParameter::TrustlessEscrow => None,
      TradeParameter::TrustedArbitration => None,
      TradeParameter::AcceptsPartialTake => None,
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TradeTimeOutLimit {
  FourDays,
  OneDay,
  NoTimeout,
  TradeEngineSpecific,
}

impl TradeTimeOutLimit {
  fn to_tag(&self) -> String {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn trade_details_times_out_parameter_to_tag() {
    let parameters = HashSet::from([TradeParameter::TradeTimesOut(TradeTimeOutLimit::OneDay)]);
    let trade_details = test_details_for_(parameters);
    let trade_parameter_tags = trade_details.parameters_to_tags();
    let expected_parameter_tags = HashSet::from(["trade-times-out-24-hours".to_string(), "trade-times-out".to_string()]);
    print!("Parameters: {:?} Expected: {:?}", trade_parameter_tags, expected_parameter_tags);
    assert_eq!(trade_parameter_tags, expected_parameter_tags);
  }

  #[test]
  fn trade_details_some_parameters_to_tags() {
    let parameters = HashSet::from([
      TradeParameter::BondsRequired,
      TradeParameter::TrustlessEscrow,
      TradeParameter::TrustedArbitration,
      TradeParameter::AcceptsPartialTake,
      TradeParameter::TradeTimesOut(TradeTimeOutLimit::FourDays),
      ]);

    let trade_details = test_details_for_(parameters);
    let trade_parameter_tags = trade_details.parameters_to_tags();

    let expected_parameter_tags = HashSet::from([
      "bonds-required".to_string(),
      "trustless-escrow".to_string(),
      "trusted-arbitration".to_string(),
      "accepts-partial-take".to_string(),
      "trade-times-out-4-days".to_string(),
      "trade-times-out".to_string(),
      ]);
      
    print!("Parameters: {:?} Expected: {:?}", trade_parameter_tags, expected_parameter_tags);
    assert_eq!(trade_parameter_tags, expected_parameter_tags);
  }

  #[test]
  fn trade_details_all_parameters_to_tags() {
    let parameters = HashSet::from([
      TradeParameter::MakerHasReputation,
      TradeParameter::TakerReputationRequired,
      TradeParameter::BondsRequired,
      TradeParameter::TrustedEscrow,
      TradeParameter::TrustedArbitration,
      TradeParameter::AcceptsPartialTake,
      TradeParameter::TradeTimesOut(TradeTimeOutLimit::TradeEngineSpecific),
      ]);

    let trade_details = test_details_for_(parameters);
    let trade_parameter_tags = trade_details.parameters_to_tags();

    let expected_parameter_tags = HashSet::from([
      "maker-has-reputation".to_string(),
      "taker-reputation-required".to_string(),
      "bonds-required".to_string(),
      "trusted-escrow".to_string(),
      "trusted-arbitration".to_string(),
      "accepts-partial-take".to_string(),
      "trade-times-out".to_string(),
      ]);
      
    print!("Parameters: {:?} Expected: {:?}", trade_parameter_tags, expected_parameter_tags);
    assert_eq!(trade_parameter_tags, expected_parameter_tags);
  }

  fn test_details_for_(parameters: HashSet<TradeParameter>) -> TradeDetails {
    let content = TradeDetailsContent {
      maker_bond_pct: None,
      taker_bond_pct: None,
      trade_timeout: None,
    };
    TradeDetails { parameters, content }
  }
}