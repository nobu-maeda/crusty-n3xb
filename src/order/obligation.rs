use super::types::*;
use crate::error::N3xbError;
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::str::FromStr;
use std::{collections::HashSet, fmt::Debug};
use strum_macros::{Display, IntoStaticStr};

#[derive(Clone, Debug)]
pub struct MakerObligation {
    pub kind: ObligationKind,
    pub content: MakerObligationContent,
}

#[derive(Clone, Debug)]
pub struct TakerObligation {
    pub kind: ObligationKind,
    pub content: TakerObligationContent,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct MakerObligationContent {
    pub amount: u64,
    pub amount_min: Option<u64>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct TakerObligationContent {
    pub limit_rate: Option<f64>,
    pub market_offset_pct: Option<f64>,
    pub market_oracles: Option<HashSet<String>>, // TODO: Change to hashset of URL type
}

#[derive(PartialEq, Clone, Display, Debug, IntoStaticStr)]
pub enum ObligationKind {
    Bitcoin(HashSet<BitcoinSettlementMethod>),
    Fiat(Currency, HashSet<FiatPaymentMethod>),
    Custom(String),
}

const OBLIGATION_KIND_SPLIT_CHAR: &str = "-";

impl ObligationKind {
    pub fn to_tags(&self) -> HashSet<String> {
        let mut tag_string_set: HashSet<String>;
        let obligation_kind_prefix_bitcoin = ObligationKind::Bitcoin([].into()).to_string();
        let obligation_kind_prefix_fiat =
            ObligationKind::Fiat(Currency::XXX, [].into()).to_string();
        let obligation_kind_prefix_custom = ObligationKind::Custom("".to_string()).to_string();

        match self {
            ObligationKind::Bitcoin(settlement_methods) => {
                let prefix_string = obligation_kind_prefix_bitcoin;
                tag_string_set = HashSet::from([prefix_string.clone()]);
                for settlment_method in settlement_methods {
                    let tag_string = format!(
                        "{}{}{}",
                        prefix_string,
                        OBLIGATION_KIND_SPLIT_CHAR,
                        settlment_method.to_string()
                    );
                    tag_string_set.insert(tag_string);
                }
            }

            ObligationKind::Fiat(currency, payment_methods) => {
                let prefix_string = obligation_kind_prefix_fiat;
                let currency_prefix_string = format!(
                    "{}{}{}",
                    prefix_string,
                    OBLIGATION_KIND_SPLIT_CHAR,
                    currency.code().to_string()
                );
                tag_string_set =
                    HashSet::from([prefix_string.to_string(), currency_prefix_string.clone()]);
                for payment_method in payment_methods {
                    let tag_string = format!(
                        "{}{}{}",
                        currency_prefix_string,
                        OBLIGATION_KIND_SPLIT_CHAR,
                        payment_method.to_string()
                    );
                    tag_string_set.insert(tag_string);
                }
            }

            ObligationKind::Custom(obligation_string) => {
                let prefix_string = obligation_kind_prefix_custom;
                tag_string_set = HashSet::from([
                    prefix_string.clone(),
                    format!(
                        "{}{}{}",
                        prefix_string, OBLIGATION_KIND_SPLIT_CHAR, obligation_string
                    ),
                ]);
            }
        }
        tag_string_set
    }

    pub fn from_tags(tags: HashSet<String>) -> Result<ObligationKind, N3xbError> {
        let obligation_kind_prefix_bitcoin = ObligationKind::Bitcoin([].into()).to_string();
        let obligation_kind_prefix_fiat =
            ObligationKind::Fiat(Currency::XXX, [].into()).to_string();
        let obligation_kind_prefix_custom = ObligationKind::Custom("".to_string()).to_string();

        let obligation_kind_prefix_set: HashSet<&str> = HashSet::from([
            obligation_kind_prefix_bitcoin.as_str(),
            obligation_kind_prefix_fiat.as_str(),
            obligation_kind_prefix_custom.as_str(),
        ]);

        let mut kind_prefix: Option<String> = None;
        let mut currency: Option<Currency> = None;
        let mut bitcoin_methods: HashSet<BitcoinSettlementMethod> = HashSet::new();
        let mut fiat_methods: HashSet<FiatPaymentMethod> = HashSet::new();
        let mut custom_ob_string: Option<String> = None;

        for tag in tags.clone() {
            let splits_set: Vec<&str> = tag.split(OBLIGATION_KIND_SPLIT_CHAR).collect();

            let splits_prefix = splits_set[0];
            if !obligation_kind_prefix_set.contains(splits_prefix) {
                return Err(N3xbError::Native(
                    "Unrecgonized Obligation Kind Prefix".to_string(),
                ));
            } else if let Some(kind_prefix_unwrapped) = &kind_prefix {
                if kind_prefix_unwrapped != splits_prefix {
                    let err_string = format!(
                        "Obligation tag set contains contradictory prefixes\n {:?}",
                        tags
                    );
                    return Err(N3xbError::Native(err_string));
                }
            } else {
                kind_prefix = Some(splits_prefix.to_string());
            }

            if &obligation_kind_prefix_bitcoin == kind_prefix.as_ref().unwrap() {
                if splits_set.len() > 1 {
                    let bitcoin_method = BitcoinSettlementMethod::from_str(splits_set[1])?;
                    bitcoin_methods.insert(bitcoin_method);
                }
            } else if &obligation_kind_prefix_fiat == kind_prefix.as_ref().unwrap() {
                if splits_set.len() > 1 {
                    currency = Some(Currency::from_str(splits_set[1])?);
                }
                if splits_set.len() > 2 {
                    let fiat_method = FiatPaymentMethod::from_str(splits_set[2])?;
                    fiat_methods.insert(fiat_method);
                }
            } else if &obligation_kind_prefix_custom == kind_prefix.as_ref().unwrap() {
                if splits_set.len() > 1 {
                    custom_ob_string = Some(splits_set[1].to_string());
                }
            } else {
                panic!("Unexpected Obligation Kind Prefix");
            }
        }

        if &obligation_kind_prefix_bitcoin == kind_prefix.as_ref().unwrap() {
            return Ok(ObligationKind::Bitcoin(bitcoin_methods));
        } else if &obligation_kind_prefix_fiat == kind_prefix.as_ref().unwrap() {
            return Ok(ObligationKind::Fiat(currency.unwrap(), fiat_methods));
        } else if &obligation_kind_prefix_custom == kind_prefix.as_ref().unwrap() {
            return Ok(ObligationKind::Custom(custom_ob_string.unwrap()));
        } else {
            panic!("Unexpected Obligation Kind");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitcoin_onchain_obligation_kind_to_tags() {
        let obligation_kind =
            ObligationKind::Bitcoin(HashSet::from([BitcoinSettlementMethod::Onchain]));
        let obligation_tags = obligation_kind.to_tags();
        let expected_tags = HashSet::from(["Bitcoin-Onchain".to_string(), "Bitcoin".to_string()]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_tags, expected_tags
        );
        assert_eq!(obligation_tags, expected_tags);
    }

    #[test]
    fn bitcoin_onchain_obligation_kind_from_tags() {
        let obligation_tags = HashSet::from(["Bitcoin-Onchain".to_string(), "Bitcoin".to_string()]);
        let obligation_kind = ObligationKind::from_tags(obligation_tags).unwrap();
        let expected_kind =
            ObligationKind::Bitcoin(HashSet::from([BitcoinSettlementMethod::Onchain]));
        print!(
            "Obligation Kind: {:?} Expected: {:?}",
            obligation_kind, expected_kind
        );
        assert_eq!(obligation_kind, expected_kind);
    }

    #[test]
    fn fiat_usd_venmo_obligation_kind_to_tags() {
        let obligation_kind = ObligationKind::Fiat(
            Currency::USD,
            HashSet::from([FiatPaymentMethod::Venmo, FiatPaymentMethod::CashApp]),
        );
        let obligation_tags = obligation_kind.to_tags();
        let expected_tags = HashSet::from([
            "Fiat-USD-Venmo".to_string(),
            "Fiat-USD-CashApp".to_string(),
            "Fiat-USD".to_string(),
            "Fiat".to_string(),
        ]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_tags, expected_tags
        );
        assert_eq!(obligation_tags, expected_tags);
    }

    #[test]
    fn fiat_usd_venmo_obligation_kind_from_tags() {
        let obligation_tags = HashSet::from([
            "Fiat-USD-Venmo".to_string(),
            "Fiat-USD-CashApp".to_string(),
            "Fiat-USD".to_string(),
            "Fiat".to_string(),
        ]);
        let obligation_kind = ObligationKind::from_tags(obligation_tags).unwrap();
        let expected_kind = ObligationKind::Fiat(
            Currency::USD,
            HashSet::from([FiatPaymentMethod::Venmo, FiatPaymentMethod::CashApp]),
        );
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_kind, expected_kind
        );
        assert_eq!(obligation_kind, expected_kind);
    }

    #[test]
    fn custom_obligation_kind_to_tags() {
        let obligation_kind = ObligationKind::Custom("Barter".to_string());
        let obligation_tags = obligation_kind.to_tags();
        let expected_tags = HashSet::from(["Custom-Barter".to_string(), "Custom".to_string()]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_tags, expected_tags
        );
        assert_eq!(obligation_tags, expected_tags);
    }

    #[test]
    fn custom_obligation_kind_from_tags() {
        let obligation_tags = HashSet::from(["Custom-Barter".to_string(), "Custom".to_string()]);
        let obligation_kind = ObligationKind::from_tags(obligation_tags).unwrap();
        let expected_kind = ObligationKind::Custom("Barter".to_string());
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_kind, expected_kind
        );
        assert_eq!(obligation_kind, expected_kind);
    }
}
