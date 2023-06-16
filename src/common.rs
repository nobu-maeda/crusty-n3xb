use serde::{Serialize, Deserialize};
use std::fmt::*;

pub enum BuySell {
  Buy,
  Sell
}

#[derive(Clone, Debug)]
pub enum BitcoinSettlementTypes {
  Onchain,
  Lightning,
}

// List of fiat payment methods from 
// https://github.com/bisq-network/bisq/blob/release/v1.9.10/core/src/main/java/bisq/core/payment/payload/PaymentMethod.java
// We are not implementing trade limits and risk association here. This should be for the higher level to determine.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FiatPaymentMethods {
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
  NEFT,  // National Electronic Funds Transfer - an electronic funds transfer system maintained by the Reserve Bank of India.
  RTGS,  // Real Time Gross Settlment
  IMPS,  // Immediate Payment Service - an instant payment inter-bank electronic funds transfer system in India
  UPI,  // Unified Payments Interface (UPI) - an instant payment system developed by National Payments Corporation of India (NPCI)
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

impl Display for FiatPaymentMethods {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
        FiatPaymentMethods::Uphold => write!(f, "Uphold"),
        FiatPaymentMethods::MoneyBeam => write!(f, "MoneyBeam"),
        FiatPaymentMethods::PopMoney => write!(f, "PopMoney"),
        FiatPaymentMethods::Revolut => write!(f, "Revolut"),
        FiatPaymentMethods::PerfectMoney => write!(f, "PerfectMoney"),
        FiatPaymentMethods::Sepa => write!(f, "SEPA"),
        FiatPaymentMethods::SepaInstant => write!(f, "SEPAInstant"),
        FiatPaymentMethods::FasterPayments => write!(f, "FasterPayments"),
        FiatPaymentMethods::NationalBank => write!(f, "NationalBank"),
        FiatPaymentMethods::JapanBank => write!(f, "JapanBank"),
        FiatPaymentMethods::AustraliaPayID => write!(f, "AustraliaPayID"),
        FiatPaymentMethods::Swish => write!(f, "Swish"),
        FiatPaymentMethods::AliPay => write!(f, "AliPay"),
        FiatPaymentMethods::WeChatPay => write!(f, "WeChatPay"),
        FiatPaymentMethods::Zelle => write!(f, "Zelle"),
        FiatPaymentMethods::InteracETransfer => write!(f, "InteracETransfer"),
        FiatPaymentMethods::USPostalMoneyOrder => write!(f, "USPostalMoneyOrder"),
        FiatPaymentMethods::CashDeposit => write!(f, "CashDeposit"),
        FiatPaymentMethods::MoneyGram => write!(f, "MoneyGram"),
        FiatPaymentMethods::WesternUnion => write!(f, "WesternUnion"),
        FiatPaymentMethods::FaceToFace => write!(f, "FaceToFace"),
        FiatPaymentMethods::HalCash => write!(f, "HalCash"),
        FiatPaymentMethods::PromptPay => write!(f, "PromptPay"),
        FiatPaymentMethods::AdvancedCash => write!(f, "AdvancedCash"),
        FiatPaymentMethods::TransferWise => write!(f, "TransferWise"),
        FiatPaymentMethods::Paysera => write!(f, "Paysera"),
        FiatPaymentMethods::Paxum => write!(f, "Paxum"),
        FiatPaymentMethods::NEFT => write!(f, "NEFT"),
        FiatPaymentMethods::RTGS => write!(f, "RTGS"),
        FiatPaymentMethods::IMPS => write!(f, "IMPS"),
        FiatPaymentMethods::UPI => write!(f, "UPI"),
        FiatPaymentMethods::Paytm => write!(f, "Paytm"),
        FiatPaymentMethods::Nequi => write!(f, "Nequi"),
        FiatPaymentMethods::Bizum => write!(f, "Bizum"),
        FiatPaymentMethods::Pix => write!(f, "Pix"),
        FiatPaymentMethods::AmazonGiftCard => write!(f, "AmazonGiftCard"),
        FiatPaymentMethods::CashByMail => write!(f, "CashByMail"),
        FiatPaymentMethods::Capitual => write!(f, "Capitual"),
        FiatPaymentMethods::Celpay => write!(f, "Celpay"),
        FiatPaymentMethods::Monese => write!(f, "Monese"),
        FiatPaymentMethods::Satispay => write!(f, "Satispay"),
        FiatPaymentMethods::Tikkie => write!(f, "Tikkie"),
        FiatPaymentMethods::Verse => write!(f, "Verse"),
        FiatPaymentMethods::Strike => write!(f, "Strike"),
        FiatPaymentMethods::SWIFT => write!(f, "SWIFT"),
        FiatPaymentMethods::ACHTransfer => write!(f, "ACHTransfer"),
        FiatPaymentMethods::DomesticWireTransfer => write!(f, "DomesticWireTransfer"),
        FiatPaymentMethods::OkPay => write!(f, "OkPay"),
        FiatPaymentMethods::CashApp => write!(f, "CashApp"),
        FiatPaymentMethods::Venmo => write!(f, "Venmo"),
    }
  }
}