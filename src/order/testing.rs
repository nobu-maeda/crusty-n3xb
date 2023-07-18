use super::{obligation::*, trade_details::*, trade_engine_details::*, types::*};
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use typetag;

pub struct SomeTestParams {}

impl SomeTestParams {
    pub fn some_uuid_string() -> String {
        "Some-UUID-String".to_string()
    }

    pub fn maker_obligation_kind() -> ObligationKind {
        ObligationKind::Fiat(
            Currency::CNY,
            HashSet::from([FiatPaymentMethod::WeChatPay, FiatPaymentMethod::AliPay]),
        )
    }

    pub fn maker_obligation_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 1000000,
            amount_min: None,
        }
    }

    pub fn taker_obligation_kind() -> ObligationKind {
        ObligationKind::Bitcoin(HashSet::from([
            BitcoinSettlementMethod::Onchain,
            BitcoinSettlementMethod::Lightning,
        ]))
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SomeTradeEngineSpecifics {
    pub test_specific_field: String,
}

#[typetag::serde(name = "test-trade-engine")]
impl TradeEngineSpecfiicsTrait for SomeTradeEngineSpecifics {}
