use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result};
use std::hash::Hash;
use strum_macros::{Display, EnumString, IntoStaticStr};

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

impl OrderTag {
    pub fn key(&self) -> String {
        let str = match self {
            OrderTag::TradeUUID(_) => "i",
            OrderTag::MakerObligations(_) => "m",
            OrderTag::TakerObligations(_) => "t",
            OrderTag::TradeDetailParameters(_) => "p",
            OrderTag::TradeEngineName(_) => "n",
            OrderTag::EventKind(_) => "k",
            OrderTag::ApplicationTag(_) => "d",
        };
        str.to_string()
    }

    pub fn key_for(tag: OrderTag) -> String {
        tag.key()
    }
}

#[derive(Clone, Debug)]
pub enum EventKind {
    MakerOrder,
}

impl Display for EventKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            EventKind::MakerOrder => write!(f, "maker-order"),
        }
    }
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
