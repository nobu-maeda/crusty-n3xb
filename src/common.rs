use serde::{Deserialize, Serialize};
use std::fmt::Result;
use std::fmt::*;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub type ArcMutex = Arc<Mutex<i32>>;

pub enum BuySell {
    Buy,
    Sell,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum BitcoinSettlementMethod {
    Onchain,
    Lightning,
}

impl Display for BitcoinSettlementMethod {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            BitcoinSettlementMethod::Onchain => write!(f, "Onchain"),
            BitcoinSettlementMethod::Lightning => write!(f, "Lightning"),
        }
    }
}

// List of fiat payment methods from
// https://github.com/bisq-network/bisq/blob/release/v1.9.10/core/src/main/java/bisq/core/payment/payload/PaymentMethod.java
// We are not implementing trade limits and risk association here. This should be for the higher level to determine.

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
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

impl Display for FiatPaymentMethod {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            FiatPaymentMethod::Uphold => write!(f, "Uphold"),
            FiatPaymentMethod::MoneyBeam => write!(f, "MoneyBeam"),
            FiatPaymentMethod::PopMoney => write!(f, "PopMoney"),
            FiatPaymentMethod::Revolut => write!(f, "Revolut"),
            FiatPaymentMethod::PerfectMoney => write!(f, "PerfectMoney"),
            FiatPaymentMethod::Sepa => write!(f, "SEPA"),
            FiatPaymentMethod::SepaInstant => write!(f, "SEPAInstant"),
            FiatPaymentMethod::FasterPayments => write!(f, "FasterPayments"),
            FiatPaymentMethod::NationalBank => write!(f, "NationalBank"),
            FiatPaymentMethod::JapanBank => write!(f, "JapanBank"),
            FiatPaymentMethod::AustraliaPayID => write!(f, "AustraliaPayID"),
            FiatPaymentMethod::Swish => write!(f, "Swish"),
            FiatPaymentMethod::AliPay => write!(f, "AliPay"),
            FiatPaymentMethod::WeChatPay => write!(f, "WeChatPay"),
            FiatPaymentMethod::Zelle => write!(f, "Zelle"),
            FiatPaymentMethod::InteracETransfer => write!(f, "InteracETransfer"),
            FiatPaymentMethod::USPostalMoneyOrder => write!(f, "USPostalMoneyOrder"),
            FiatPaymentMethod::CashDeposit => write!(f, "CashDeposit"),
            FiatPaymentMethod::MoneyGram => write!(f, "MoneyGram"),
            FiatPaymentMethod::WesternUnion => write!(f, "WesternUnion"),
            FiatPaymentMethod::FaceToFace => write!(f, "FaceToFace"),
            FiatPaymentMethod::HalCash => write!(f, "HalCash"),
            FiatPaymentMethod::PromptPay => write!(f, "PromptPay"),
            FiatPaymentMethod::AdvancedCash => write!(f, "AdvancedCash"),
            FiatPaymentMethod::TransferWise => write!(f, "TransferWise"),
            FiatPaymentMethod::Paysera => write!(f, "Paysera"),
            FiatPaymentMethod::Paxum => write!(f, "Paxum"),
            FiatPaymentMethod::NEFT => write!(f, "NEFT"),
            FiatPaymentMethod::RTGS => write!(f, "RTGS"),
            FiatPaymentMethod::IMPS => write!(f, "IMPS"),
            FiatPaymentMethod::UPI => write!(f, "UPI"),
            FiatPaymentMethod::Paytm => write!(f, "Paytm"),
            FiatPaymentMethod::Nequi => write!(f, "Nequi"),
            FiatPaymentMethod::Bizum => write!(f, "Bizum"),
            FiatPaymentMethod::Pix => write!(f, "Pix"),
            FiatPaymentMethod::AmazonGiftCard => write!(f, "AmazonGiftCard"),
            FiatPaymentMethod::CashByMail => write!(f, "CashByMail"),
            FiatPaymentMethod::Capitual => write!(f, "Capitual"),
            FiatPaymentMethod::Celpay => write!(f, "Celpay"),
            FiatPaymentMethod::Monese => write!(f, "Monese"),
            FiatPaymentMethod::Satispay => write!(f, "Satispay"),
            FiatPaymentMethod::Tikkie => write!(f, "Tikkie"),
            FiatPaymentMethod::Verse => write!(f, "Verse"),
            FiatPaymentMethod::Strike => write!(f, "Strike"),
            FiatPaymentMethod::SWIFT => write!(f, "SWIFT"),
            FiatPaymentMethod::ACHTransfer => write!(f, "ACHTransfer"),
            FiatPaymentMethod::DomesticWireTransfer => write!(f, "DomesticWireTransfer"),
            FiatPaymentMethod::OkPay => write!(f, "OkPay"),
            FiatPaymentMethod::CashApp => write!(f, "CashApp"),
            FiatPaymentMethod::Venmo => write!(f, "Venmo"),
        }
    }
}
