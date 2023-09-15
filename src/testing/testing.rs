use iso_currency::Currency;
use secp256k1::{PublicKey, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

use std::any::Any;
use std::collections::HashSet;
use std::str::FromStr;

use crate::common::types::*;
use crate::offer::Obligation;
use crate::order::*;

pub struct SomeTestParams {}

impl SomeTestParams {
    pub fn some_secret_key() -> SecretKey {
        SecretKey::from_str("01010101010101010001020304050607ffff0000ffff00006363636363636363")
            .unwrap()
    }

    pub fn some_x_only_public_key() -> XOnlyPublicKey {
        let kpk = PublicKey::from_str(
            "02e6642fd69bd211f93f7f1f36ca51a26a5290eb2dd1b0d8279a87bb0d480c8443",
        )
        .unwrap();
        XOnlyPublicKey::from(kpk)
    }

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
        "{\"maker_obligation\":{\"amount\":1000000,\"amount_min\":null},\"taker_obligation\":{\"limit_rate\":1e-6,\"market_offset_pct\":null,\"market_oracles\":null},\"trade_details\":{\"maker_bond_pct\":null,\"taker_bond_pct\":null,\"trade_timeout\":null},\"trade_engine_specifics\":{\"type\":\"some_trade_engine_maker_order_specifics\",\"test_specific_field\":\"some-test-specific-info\"},\"pow_difficulty\":8}".to_string()
    }

    pub fn offer_maker_obligation() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            amount: 1000000,
            bond_amount: Some(100000),
        }
    }

    pub fn offer_taker_obligation() -> Obligation {
        Obligation {
            kind: ObligationKind::Bitcoin(BitcoinSettlementMethod::Onchain),
            amount: 1,
            bond_amount: Some(100000),
        }
    }

    pub fn offer_marker_oracle_used() -> Option<String> {
        None
    }

    pub fn offer_pow_difficulty() -> Option<u64> {
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SomeTradeEngineMakerOrderSpecifics {
    pub test_specific_field: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SomeTradeEngineTakerOfferSpecifics {
    pub test_specific_field: String,
}

#[typetag::serde(name = "some_trade_engine_maker_order_specifics")]
impl SerdeGenericTrait for SomeTradeEngineMakerOrderSpecifics {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}

#[typetag::serde(name = "some_trade_engine_taker_offer_specifics")]
impl SerdeGenericTrait for SomeTradeEngineTakerOfferSpecifics {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
