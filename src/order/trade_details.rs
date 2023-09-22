use crate::common::error::N3xbError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::result::Result;
use std::str::FromStr;
use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradeDetails {
    pub parameters: HashSet<TradeParameter>,
    pub content: TradeDetailsContent,
}

impl TradeDetails {
    pub fn parameters_to_tags(parameters: HashSet<TradeParameter>) -> HashSet<String> {
        let mut tag_string_set: HashSet<String> = HashSet::new();
        for parameter in parameters.iter() {
            tag_string_set.insert(parameter.to_string());
            if parameter.to_tag() != parameter.to_string() {
                tag_string_set.insert(parameter.to_tag());
            }
        }
        tag_string_set
    }

    pub fn tags_to_parameters(tags: HashSet<String>) -> HashSet<TradeParameter> {
        let mut parameters_set: HashSet<TradeParameter> = HashSet::new();
        let mut parse_failed_tags: Vec<N3xbError> = Vec::new(); // TODO: What do we do with errors when parsing some tags?

        for tag in tags {
            let some_parameter = match TradeParameter::from_tag(&tag) {
                Ok(parameter) => parameter,
                Err(_) => {
                    // TODO: What do we do with the prior error?
                    parse_failed_tags.push(N3xbError::TagParsing(tag.clone()));
                    None
                }
            };

            if let Some(parameter) = some_parameter {
                parameters_set.insert(parameter);
            }
        }
        parameters_set
    }
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TradeDetailsContent {
    pub maker_bond_pct: Option<u32>,
    pub taker_bond_pct: Option<u32>,
    pub trade_timeout: Option<u32>,
}

#[derive(
    Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, EnumString, Display, IntoStaticStr,
)]
pub enum TradeParameter {
    MakerHasReputation,
    TakerReputationRequired,
    BondsRequired,
    TrustedEscrow,
    TrustlessEscrow,
    TrustedArbitration,
    AcceptsPartialTake,
    TradeTimesOut(TradeTimeOutLimit),
}

const OBLIGATION_KIND_SPLIT_CHAR: &str = "-";

impl TradeParameter {
    fn to_tag(&self) -> String {
        match self {
            TradeParameter::TradeTimesOut(timelimit) => {
                format!(
                    "{}{}{}",
                    self.to_string(),
                    OBLIGATION_KIND_SPLIT_CHAR,
                    timelimit.to_string()
                )
            }
            _ => self.to_string(),
        }
    }

    fn from_tag(tag_string: &str) -> Result<Option<Self>, N3xbError> {
        let trade_parameter_trade_times_out_prefix =
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::default()).to_string();
        let splits_set: Vec<&str> = tag_string.split(OBLIGATION_KIND_SPLIT_CHAR).collect();

        if splits_set[0] == trade_parameter_trade_times_out_prefix {
            if splits_set.len() > 1 {
                let timeout_limit = TradeTimeOutLimit::from_str(splits_set[1])?;
                return Ok(Some(TradeParameter::TradeTimesOut(timeout_limit)));
            } else {
                return Ok(None);
            }
        } else {
            let parameter = TradeParameter::from_str(splits_set[0])?;
            return Ok(Some(parameter));
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Debug,
    Default,
    EnumString,
    Display,
    IntoStaticStr,
)]
pub enum TradeTimeOutLimit {
    #[default]
    NoTimeout,
    OneDay,
    FourDays,
    TradeEngineSpecific,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trade_details_times_out_parameter_to_tag() {
        let parameters = HashSet::from([TradeParameter::TradeTimesOut(TradeTimeOutLimit::OneDay)]);
        let trade_details = test_details_for_(parameters);
        let trade_parameter_tags = TradeDetails::parameters_to_tags(trade_details.parameters);
        let expected_parameter_tags = HashSet::from([
            "TradeTimesOut-OneDay".to_string(),
            "TradeTimesOut".to_string(),
        ]);
        print!(
            "Parameters: {:?} Expected: {:?}",
            trade_parameter_tags, expected_parameter_tags
        );
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
        let trade_parameter_tags = TradeDetails::parameters_to_tags(trade_details.parameters);

        let expected_parameter_tags = HashSet::from([
            "BondsRequired".to_string(),
            "TrustlessEscrow".to_string(),
            "TrustedArbitration".to_string(),
            "AcceptsPartialTake".to_string(),
            "TradeTimesOut-FourDays".to_string(),
            "TradeTimesOut".to_string(),
        ]);

        print!(
            "Parameters: {:?} Expected: {:?}",
            trade_parameter_tags, expected_parameter_tags
        );
        assert_eq!(trade_parameter_tags, expected_parameter_tags);
    }

    #[test]
    fn trade_details_some_parameters_from_tags() {
        let parameter_tags = HashSet::from([
            "BondsRequired".to_string(),
            "TrustlessEscrow".to_string(),
            "TrustedArbitration".to_string(),
            "AcceptsPartialTake".to_string(),
            "TradeTimesOut-FourDays".to_string(),
            "TradeTimesOut".to_string(),
        ]);

        let expected_parameters = HashSet::from([
            TradeParameter::BondsRequired,
            TradeParameter::TrustlessEscrow,
            TradeParameter::TrustedArbitration,
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::FourDays),
        ]);

        let parameters = TradeDetails::tags_to_parameters(parameter_tags);

        print!(
            "Parameters: {:?} Expected: {:?}",
            parameters, expected_parameters
        );
        assert_eq!(parameters, expected_parameters);
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
        let trade_parameter_tags = TradeDetails::parameters_to_tags(trade_details.parameters);

        let expected_parameter_tags = HashSet::from([
            "MakerHasReputation".to_string(),
            "TakerReputationRequired".to_string(),
            "BondsRequired".to_string(),
            "TrustedEscrow".to_string(),
            "TrustedArbitration".to_string(),
            "AcceptsPartialTake".to_string(),
            "TradeTimesOut".to_string(),
            "TradeTimesOut-TradeEngineSpecific".to_string(),
        ]);

        print!(
            "Parameters: {:?} Expected: {:?}",
            trade_parameter_tags, expected_parameter_tags
        );
        assert_eq!(trade_parameter_tags, expected_parameter_tags);
    }

    #[test]
    fn trade_details_all_parameters_from_tags() {
        let parameter_tags = HashSet::from([
            "MakerHasReputation".to_string(),
            "TakerReputationRequired".to_string(),
            "BondsRequired".to_string(),
            "TrustedEscrow".to_string(),
            "TrustedArbitration".to_string(),
            "AcceptsPartialTake".to_string(),
            "TradeTimesOut".to_string(),
            "TradeTimesOut-TradeEngineSpecific".to_string(),
        ]);

        let expected_parameters = HashSet::from([
            TradeParameter::MakerHasReputation,
            TradeParameter::TakerReputationRequired,
            TradeParameter::BondsRequired,
            TradeParameter::TrustedEscrow,
            TradeParameter::TrustedArbitration,
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::TradeEngineSpecific),
        ]);

        let parameters = TradeDetails::tags_to_parameters(parameter_tags);

        print!(
            "Parameters: {:?} Expected: {:?}",
            parameters, expected_parameters
        );
        assert_eq!(parameters, expected_parameters);
    }

    fn test_details_for_(parameters: HashSet<TradeParameter>) -> TradeDetails {
        let content = TradeDetailsContent {
            maker_bond_pct: None,
            taker_bond_pct: None,
            trade_timeout: None,
        };
        TradeDetails {
            parameters,
            content,
        }
    }
}
