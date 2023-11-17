use dyn_clone::DynClone;
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};

use std::any::Any;
use std::hash::Hash;
use std::{collections::HashSet, fmt::Debug, str::FromStr};

use crate::common::error::N3xbError;

pub type EventIdString = String;

pub enum BuySell {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum SerdeGenericType {
    TakerOffer,
    TradeResponse,
    TradeEngineSpecific,
}

#[typetag::serde(tag = "type")]
pub trait SerdeGenericTrait: DynClone + Debug + Send {
    fn any_ref(&self) -> &dyn Any;
}

dyn_clone::clone_trait_object!(SerdeGenericTrait);

impl dyn SerdeGenericTrait {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.any_ref().downcast_ref()
    }
}

#[derive(
    Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, EnumString, Display, IntoStaticStr,
)]
pub enum BitcoinSettlementMethod {
    Onchain,
    Lightning,
}

// List of fiat payment methods from
// https://github.com/bisq-network/bisq/blob/release/v1.9.10/core/src/main/java/bisq/core/payment/payload/PaymentMethod.java
// We are not implementing trade limits and risk association here. This should be for the higher level to determine.

#[derive(
    PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, EnumString, Display, IntoStaticStr,
)]
pub enum FiatPaymentMethod {
    Uphold,
    MoneyBeam,
    PopMoney,
    Revolut,
    PerfectMoney,
    Sepa,
    SepaInstant,
    FasterPayments,
    NationalBank,
    JapanBank,
    AustraliaPayID,
    // SameBank, ??
    // SpecificBanks, ??
    Swish,
    AliPay,
    WeChatPay,
    // ClearXchange, changed to Zelle,
    // ChaseQuickPay, changed to Zelle,
    Zelle,
    InteracETransfer,
    USPostalMoneyOrder,
    CashDeposit,
    MoneyGram,
    WesternUnion,
    FaceToFace,
    HalCash,
    // Blockchains, ??
    PromptPay,
    AdvancedCash,
    TransferWise,
    // TransferWiseUSD, Why is this separate from the TransferWise method?
    Paysera,
    Paxum,
    NEFT, // National Electronic Funds Transfer - an electronic funds transfer system maintained by the Reserve Bank of India.
    RTGS, // Real Time Gross Settlment
    IMPS, // Immediate Payment Service - an instant payment inter-bank electronic funds transfer system in India
    UPI, // Unified Payments Interface (UPI) - an instant payment system developed by National Payments Corporation of India (NPCI)
    Paytm,
    Nequi,
    Bizum,
    Pix,
    AmazonGiftCard,
    // BlockchainsInstant, ?? and why is this different from just Blockchains? Atomic Swap?
    CashByMail,
    Capitual,
    Celpay,
    Monese,
    Satispay,
    Tikkie,
    Verse,
    Strike,
    SWIFT,
    ACHTransfer,
    DomesticWireTransfer,
    OkPay,
    CashApp,
    Venmo,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Display, Deserialize, Serialize, IntoStaticStr)]
pub enum ObligationKind {
    Bitcoin(Option<BitcoinSettlementMethod>),
    Fiat(Currency, Option<FiatPaymentMethod>),
    Custom(String),
}

const OBLIGATION_KIND_SPLIT_CHAR: &str = "-";

impl ObligationKind {
    pub fn is_bitcoin(&self) -> bool {
        match self {
            ObligationKind::Bitcoin(_) => true,
            ObligationKind::Fiat(_, _) => false,
            ObligationKind::Custom(_) => false,
        }
    }

    pub fn is_same_currency_as(&self, kind: ObligationKind) -> bool {
        match self {
            ObligationKind::Bitcoin(_) => match kind {
                ObligationKind::Bitcoin(_) => true,
                ObligationKind::Fiat(_, _) => false,
                ObligationKind::Custom(_) => false,
            },
            ObligationKind::Fiat(self_currency, _) => match kind {
                ObligationKind::Bitcoin(_) => false,
                ObligationKind::Fiat(kind_currency, _) => self_currency.to_owned() == kind_currency,
                ObligationKind::Custom(_) => false,
            },
            ObligationKind::Custom(self_custom) => match kind {
                ObligationKind::Bitcoin(_) => false,
                ObligationKind::Fiat(_, _) => false,
                ObligationKind::Custom(kind_custom) => self_custom.to_owned() == kind_custom,
            },
        }
    }

    pub fn to_tag_strings(&self) -> HashSet<String> {
        let mut tag_string_set: HashSet<String>;
        let obligation_kind_prefix_bitcoin = ObligationKind::Bitcoin(None).to_string();
        let obligation_kind_prefix_fiat = ObligationKind::Fiat(Currency::XXX, None).to_string();
        let obligation_kind_prefix_custom = ObligationKind::Custom("".to_string()).to_string();

        match self {
            ObligationKind::Bitcoin(settlement_method) => {
                let prefix_string = obligation_kind_prefix_bitcoin;
                tag_string_set = HashSet::from([prefix_string.clone()]);

                if let Some(settlement_method) = settlement_method {
                    let tag_string = format!(
                        "{}{}{}",
                        prefix_string,
                        OBLIGATION_KIND_SPLIT_CHAR,
                        settlement_method.to_string()
                    );
                    tag_string_set.insert(tag_string);
                }
            }

            ObligationKind::Fiat(currency, payment_method) => {
                let prefix_string = obligation_kind_prefix_fiat;
                let currency_prefix_string = format!(
                    "{}{}{}",
                    prefix_string,
                    OBLIGATION_KIND_SPLIT_CHAR,
                    currency.code().to_string()
                );
                tag_string_set =
                    HashSet::from([prefix_string.to_string(), currency_prefix_string.clone()]);

                if let Some(payment_method) = payment_method {
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

    pub fn from_tag_strings(tags: HashSet<String>) -> Result<HashSet<ObligationKind>, N3xbError> {
        let obligation_kind_prefix_bitcoin = ObligationKind::Bitcoin(None).to_string();
        let obligation_kind_prefix_fiat = ObligationKind::Fiat(Currency::XXX, None).to_string();
        let obligation_kind_prefix_custom = ObligationKind::Custom("".to_string()).to_string();

        let obligation_kind_prefix_set: HashSet<&str> = HashSet::from([
            obligation_kind_prefix_bitcoin.as_str(),
            obligation_kind_prefix_fiat.as_str(),
            obligation_kind_prefix_custom.as_str(),
        ]);

        let mut kind_prefix: Option<String> = None;
        let mut currency: Option<Currency> = None;
        let mut obligation_kinds: HashSet<ObligationKind> = HashSet::new();

        for tag in tags.clone() {
            let splits_set: Vec<&str> = tag.split(OBLIGATION_KIND_SPLIT_CHAR).collect();

            let splits_prefix = splits_set[0];
            if !obligation_kind_prefix_set.contains(splits_prefix) {
                return Err(N3xbError::Simple(
                    "Unrecgonized Obligation Kind Prefix".to_string(),
                ));

            // Checking if all obligations are of the same kind is actually optional after refactor
            } else if let Some(kind_prefix_unwrapped) = &kind_prefix {
                if kind_prefix_unwrapped != splits_prefix {
                    let err_string = format!(
                        "Obligation tag set contains contradictory prefixes\n {:?}",
                        tags
                    );
                    return Err(N3xbError::Simple(err_string));
                }
            } else {
                kind_prefix = Some(splits_prefix.to_string());
            }

            if &obligation_kind_prefix_bitcoin == kind_prefix.as_ref().unwrap() {
                if splits_set.len() > 1 {
                    let bitcoin_method = BitcoinSettlementMethod::from_str(splits_set[1])?;
                    obligation_kinds.insert(ObligationKind::Bitcoin(Some(bitcoin_method)));
                }
            } else if &obligation_kind_prefix_fiat == kind_prefix.as_ref().unwrap() {
                if splits_set.len() > 1 {
                    currency = Some(Currency::from_str(splits_set[1])?);
                }
                if splits_set.len() > 2 {
                    let fiat_method = FiatPaymentMethod::from_str(splits_set[2])?;
                    obligation_kinds
                        .insert(ObligationKind::Fiat(currency.unwrap(), Some(fiat_method)));
                }
            } else if &obligation_kind_prefix_custom == kind_prefix.as_ref().unwrap() {
                if splits_set.len() > 1 {
                    obligation_kinds.insert(ObligationKind::Custom(splits_set[1].to_string()));
                }
            } else {
                panic!("Unexpected Obligation Kind Prefix");
            }
        }
        Ok(obligation_kinds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usd_venmo_is_same_currency_usd_cashapp() {
        let kind1 = ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::Venmo));
        let kind2 = ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::CashApp));
        assert!(kind1.is_same_currency_as(kind2));
    }

    #[test]
    fn aud_wise_is_not_same_currency_as_gbp_wise() {
        let kind1 = ObligationKind::Fiat(Currency::AUD, Some(FiatPaymentMethod::TransferWise));
        let kind2 = ObligationKind::Fiat(Currency::GBP, Some(FiatPaymentMethod::TransferWise));
        assert!(!kind1.is_same_currency_as(kind2));
    }

    #[test]
    fn custom_barter_is_same_currency_as_custom_barter() {
        let kind1 = ObligationKind::Custom("barter".to_string());
        let kind2 = ObligationKind::Custom("barter".to_string());
        assert!(kind1.is_same_currency_as(kind2));
    }

    #[test]
    fn custom_barter_is_not_same_currency_as_custom_charity() {
        let kind1 = ObligationKind::Custom("barter".to_string());
        let kind2 = ObligationKind::Custom("charity".to_string());
        assert!(!kind1.is_same_currency_as(kind2));
    }

    #[test]
    fn bitcoin_onchain_obligation_kind_to_tags() {
        let obligation_kind = ObligationKind::Bitcoin(Some(BitcoinSettlementMethod::Onchain));
        let obligation_tags = obligation_kind.to_tag_strings();
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
        let obligation_kinds = ObligationKind::from_tag_strings(obligation_tags).unwrap();
        let expected_kinds = HashSet::from([ObligationKind::Bitcoin(Some(
            BitcoinSettlementMethod::Onchain,
        ))]);
        print!(
            "Obligation Kind: {:?} Expected: {:?}",
            obligation_kinds, expected_kinds
        );
        assert_eq!(obligation_kinds, expected_kinds);
    }

    #[test]
    fn fiat_usd_venmo_obligation_kind_to_tags() {
        let obligation_kinds = HashSet::from([
            ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::Venmo)),
            ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::CashApp)),
        ]);
        let obligation_tags: HashSet<String> = obligation_kinds
            .iter()
            .flat_map(|k| k.to_tag_strings())
            .collect();
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
        let obligation_kinds = ObligationKind::from_tag_strings(obligation_tags).unwrap();
        let expected_kinds = HashSet::from([
            ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::Venmo)),
            ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::CashApp)),
        ]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_kinds, expected_kinds
        );
        assert_eq!(obligation_kinds, expected_kinds);
    }

    #[test]
    fn custom_obligation_kind_to_tags() {
        let obligation_kind = ObligationKind::Custom("Barter".to_string());
        let obligation_tags = obligation_kind.to_tag_strings();
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
        let obligation_kinds = ObligationKind::from_tag_strings(obligation_tags).unwrap();
        let expected_kinds = HashSet::from([ObligationKind::Custom("Barter".to_string())]);
        print!(
            "Obligation: {:?} Expected: {:?}",
            obligation_kinds, expected_kinds
        );
        assert_eq!(obligation_kinds, expected_kinds);
    }
}
