use serde::{Serialize, Deserialize};
use iso_currency::Currency;

use std::fmt::*;

use crate::common::*;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MakerObligationContent {
  pub amount: u64,
  pub amount_min: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TakerObligationContent{
  pub limit_rate: Option<f64>,
  pub market_offset_pct: Option<f64>,
  pub market_oracles: Option<Vec<String>>,  // TODO: Change to vector of URL type
}

#[derive(Clone, Debug)]
pub enum ObligationKind {
  Bitcoin(Vec<BitcoinSettlementMethod>),
  Fiat(Currency, Vec<FiatPaymentMethod>),
  Custom(String),
}

impl ObligationKind {
  pub fn to_tags(&self) -> Vec<String> {
    let mut tag_string_vec: Vec<String>;
    match self {
      ObligationKind::Bitcoin(settlement_methods) => {
        let prefix_string = "ob-bitcoin";
        tag_string_vec = vec![prefix_string.to_string()];
        for settlment_method in settlement_methods {
          let tag_string = format!("{}-{}", prefix_string, settlment_method.to_string());
          tag_string_vec.push(tag_string.to_lowercase());
        }
      }

      ObligationKind::Fiat(currency, payment_methods) => {
        let prefix_string = "ob-fiat";
        let currency_prefix_string = format!("{}-{}", prefix_string, currency.code().to_string());
        tag_string_vec = vec![prefix_string.to_string(), currency_prefix_string.to_lowercase()];
        for payment_method in payment_methods {
          let tag_string = format!("{}-{}", currency_prefix_string, payment_method.to_string());
          tag_string_vec.push(tag_string.to_lowercase());
        }
      }

      ObligationKind::Custom(obligation_string) => {
        let prefix_string = "ob-custom";
        tag_string_vec = vec![prefix_string.to_string(), format!("{}-{}", prefix_string, obligation_string).to_lowercase()];
      }
    }
    tag_string_vec
  }
}

impl Display for ObligationKind {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      ObligationKind::Bitcoin(settlement_methods) => 
        write!(f, "Bitcoin {:?} Settlements", settlement_methods),
      ObligationKind::Fiat(currency, payment_methods) => 
        write!(f, "Fiat {:?} {:?} Settlements", currency, payment_methods),
      ObligationKind::Custom(obligation_string) => 
        write!(f, "Custom Settlement {:?}", obligation_string),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bitcoin_onchain_obligation_kind_to_tags() {
    let obligation_kind = ObligationKind::Bitcoin(vec![BitcoinSettlementMethod::Onchain]);
    let mut obligation_tags = obligation_kind.to_tags();
    obligation_tags.sort();
    let mut expected_tags = vec!["ob-bitcoin-onchain", "ob-bitcoin"];
    expected_tags.sort();
    assert_eq!(obligation_tags, expected_tags);
  }

  #[test]
  fn fiat_usd_venmo_obligation_kind_to_tags() {
    let obligation_kind = ObligationKind::Fiat(Currency::USD, vec![FiatPaymentMethod::Venmo, FiatPaymentMethod::CashApp]);
    let mut obligation_tags = obligation_kind.to_tags();
    obligation_tags.sort();
    let mut expected_tags = vec!["ob-fiat-usd-venmo", "ob-fiat-usd-cashapp", "ob-fiat-usd", "ob-fiat"];
    expected_tags.sort();
    assert_eq!(obligation_tags, expected_tags);
  }

  #[test]
  fn custom_obligation_kind_to_tags() {
    let obligation_kind = ObligationKind::Custom("barter".to_string());
    let mut obligation_tags = obligation_kind.to_tags();
    obligation_tags.sort();
    let mut expected_tags = vec!["ob-custom-barter", "ob-custom"];
    expected_tags.sort();
    assert_eq!(obligation_tags, expected_tags);
  }
}