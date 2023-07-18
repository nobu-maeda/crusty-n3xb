use super::types::*;
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::*};

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

#[derive(PartialEq, Clone, Debug)]
pub enum ObligationKind {
    Bitcoin(HashSet<BitcoinSettlementMethod>),
    Fiat(Currency, HashSet<FiatPaymentMethod>),
    Custom(String),
}

impl ObligationKind {
    pub fn to_tags(&self) -> HashSet<String> {
        let mut tag_string_set: HashSet<String>;
        match self {
            ObligationKind::Bitcoin(settlement_methods) => {
                let prefix_string = "ob-bitcoin";
                tag_string_set = HashSet::from([prefix_string.to_string()]);
                for settlment_method in settlement_methods {
                    let tag_string = format!("{}-{}", prefix_string, settlment_method.to_string());
                    tag_string_set.insert(tag_string.to_lowercase());
                }
            }

            ObligationKind::Fiat(currency, payment_methods) => {
                let prefix_string = "ob-fiat";
                let currency_prefix_string =
                    format!("{}-{}", prefix_string, currency.code().to_string());
                tag_string_set = HashSet::from([
                    prefix_string.to_string(),
                    currency_prefix_string.to_lowercase(),
                ]);
                for payment_method in payment_methods {
                    let tag_string =
                        format!("{}-{}", currency_prefix_string, payment_method.to_string());
                    tag_string_set.insert(tag_string.to_lowercase());
                }
            }

            ObligationKind::Custom(obligation_string) => {
                let prefix_string = "ob-custom";
                tag_string_set = HashSet::from([
                    prefix_string.to_string(),
                    format!("{}-{}", prefix_string, obligation_string).to_lowercase(),
                ]);
            }
        }
        tag_string_set
    }
}

impl Display for ObligationKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ObligationKind::Bitcoin(settlement_methods) => {
                write!(f, "Bitcoin {:?} Settlements", settlement_methods)
            }
            ObligationKind::Fiat(currency, payment_methods) => {
                write!(f, "Fiat {:?} {:?} Settlements", currency, payment_methods)
            }
            ObligationKind::Custom(obligation_string) => {
                write!(f, "Custom Settlement {:?}", obligation_string)
            }
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
        let expected_tags =
            HashSet::from(["ob-bitcoin-onchain".to_string(), "ob-bitcoin".to_string()]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_tags, expected_tags
        );
        assert_eq!(obligation_tags, expected_tags);
    }

    #[test]
    fn fiat_usd_venmo_obligation_kind_to_tags() {
        let obligation_kind = ObligationKind::Fiat(
            Currency::USD,
            HashSet::from([FiatPaymentMethod::Venmo, FiatPaymentMethod::CashApp]),
        );
        let obligation_tags = obligation_kind.to_tags();
        let expected_tags = HashSet::from([
            "ob-fiat-usd-venmo".to_string(),
            "ob-fiat-usd-cashapp".to_string(),
            "ob-fiat-usd".to_string(),
            "ob-fiat".to_string(),
        ]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_tags, expected_tags
        );
        assert_eq!(obligation_tags, expected_tags);
    }

    #[test]
    fn custom_obligation_kind_to_tags() {
        let obligation_kind = ObligationKind::Custom("barter".to_string());
        let obligation_tags = obligation_kind.to_tags();
        let expected_tags =
            HashSet::from(["ob-custom-barter".to_string(), "ob-custom".to_string()]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_tags, expected_tags
        );
        assert_eq!(obligation_tags, expected_tags);
    }
}
