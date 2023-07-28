use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result};
use std::hash::Hash;
use std::{collections::HashSet, str::FromStr};
use strum_macros::{Display, EnumString, IntoStaticStr};

use crate::error::N3xbError;

pub enum BuySell {
    Buy,
    Sell,
}

#[derive(Clone, Debug)]
pub enum OrderTag {
    TradeUUID(String),
    MakerObligations(HashSet<String>),
    TakerObligations(HashSet<String>),
    TradeDetailParameters(HashSet<String>),
    TradeEngineName(String),
    EventKind(EventKind),
    ApplicationTag(String),
}

const ORDER_TAG_TRADE_UUID_KEY: &str = "i";
const ORDER_TAG_MAKER_OBLIGATIONS_KEY: &str = "m";
const ORDER_TAG_TAKER_OBLIGATIONS_KEY: &str = "t";
const ORDER_TAG_TRADE_DETAIL_PARAMETERS_KEY: &str = "p";
const ORDER_TAG_TRADE_ENGINE_NAME_KEY: &str = "n";
const ORDER_TAG_EVENT_KIND_KEY: &str = "k";
const ORDER_TAG_APPLICATION_TAG_KEY: &str = "d";

impl OrderTag {
    pub fn key(&self) -> String {
        let str = match self {
            OrderTag::TradeUUID(_) => ORDER_TAG_TRADE_UUID_KEY,
            OrderTag::MakerObligations(_) => ORDER_TAG_MAKER_OBLIGATIONS_KEY,
            OrderTag::TakerObligations(_) => ORDER_TAG_TAKER_OBLIGATIONS_KEY,
            OrderTag::TradeDetailParameters(_) => ORDER_TAG_TRADE_DETAIL_PARAMETERS_KEY,
            OrderTag::TradeEngineName(_) => ORDER_TAG_TRADE_ENGINE_NAME_KEY,
            OrderTag::EventKind(_) => ORDER_TAG_EVENT_KIND_KEY,
            OrderTag::ApplicationTag(_) => ORDER_TAG_APPLICATION_TAG_KEY,
        };
        str.to_string()
    }

    pub fn key_for(tag: OrderTag) -> String {
        tag.key()
    }
}

    pub fn from_key(key: String, value: Vec<String>) -> Result<Self, N3xbError> {
        match key.as_str() {
            ORDER_TAG_TRADE_UUID_KEY => Ok(OrderTag::TradeUUID(value[0].clone())),
            ORDER_TAG_MAKER_OBLIGATIONS_KEY => {
                let tag_set: HashSet<String> = HashSet::from_iter(value);
                Ok(OrderTag::MakerObligations(tag_set))
            }
            ORDER_TAG_TAKER_OBLIGATIONS_KEY => {
                Ok(OrderTag::TakerObligations(HashSet::from_iter(value)))
            }
            ORDER_TAG_TRADE_DETAIL_PARAMETERS_KEY => {
                Ok(OrderTag::TradeDetailParameters(HashSet::from_iter(value)))
            }
            ORDER_TAG_TRADE_ENGINE_NAME_KEY => Ok(OrderTag::TradeEngineName(value[0].clone())),
            ORDER_TAG_EVENT_KIND_KEY => {
                let event_kind = EventKind::from_str(value[0].as_str())?;
                Ok(OrderTag::EventKind(event_kind))
            }
            ORDER_TAG_APPLICATION_TAG_KEY => Ok(OrderTag::ApplicationTag(value[0].clone())),
            _ => Err(N3xbError::Simple(format!(
                "Unrecognized key '{}' for Order Tag",
                key
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
pub enum EventKind {
    MakerOrder,
}

pub static N3XB_APPLICATION_TAG: &str = "n3xb";

#[derive(PartialEq, Eq, Hash, Clone, Debug, EnumString, Display, IntoStaticStr)]
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
