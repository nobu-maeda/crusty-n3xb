use iso_currency::Currency;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::common::types::*;
use crate::order::*;

pub struct SomeTestParams {}

impl SomeTestParams {
    pub fn some_uuid_string() -> String {
        "Some-UUID-String".to_string()
    }

    pub fn maker_obligation_kinds() -> HashSet<ObligationKind> {
        HashSet::from([
            ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::AliPay),
        ])
    }

    pub fn maker_obligation_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 1000000,
            amount_min: None,
        }
    }

    pub fn taker_obligation_kinds() -> HashSet<ObligationKind> {
        HashSet::from([
            ObligationKind::Bitcoin(BitcoinSettlementMethod::Onchain),
            ObligationKind::Bitcoin(BitcoinSettlementMethod::Lightning),
        ])
    }

    pub fn taker_obligation_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(0.000001),
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn trade_parameters() -> HashSet<TradeParameter> {
        HashSet::from([
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::NoTimeout),
        ])
    }

    pub fn trade_details_content() -> TradeDetailsContent {
        TradeDetailsContent {
            maker_bond_pct: None,
            taker_bond_pct: None,
            trade_timeout: None,
        }
    }

    pub fn engine_name_str() -> String {
        "some-trade-mechanics".to_string()
    }

    pub fn engine_specific_str() -> String {
        "some-test-specific-info".to_string()
    }

    pub fn pow_difficulty() -> u64 {
        8u64
    }

    pub fn expected_json_string() -> String {
        "{\"maker_obligation\":{\"amount\":1000000,\"amount_min\":null},\"taker_obligation\":{\"limit_rate\":1e-6,\"market_offset_pct\":null,\"market_oracles\":null},\"trade_details\":{\"maker_bond_pct\":null,\"taker_bond_pct\":null,\"trade_timeout\":null},\"trade_engine_specifics\":{\"test_specific_field\":\"some-test-specific-info\"},\"pow_difficulty\":8}".to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SomeTradeEngineMakerOrderSpecifics {
    pub test_specific_field: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SomeTradeEngineTakerOfferSpecifics {
    pub test_specific_field: String,
}

impl SerdeGenericTrait for SomeTradeEngineMakerOrderSpecifics {}
impl SerdeGenericTrait for SomeTradeEngineTakerOfferSpecifics {}
