use std::any::Any;
use std::collections::HashSet;
use std::str::FromStr;

use iso_currency::Currency;
use secp256k1::{PublicKey, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::*;
use crate::order::*;

use super::SomeTestParams;

pub struct SomeTestOrderParams {}

impl SomeTestOrderParams {
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

    pub fn some_uuid() -> Uuid {
        Uuid::from_str("20c38e4b-037b-4654-b99c-1d9f2beb755f").unwrap()
    }

    pub fn maker_obligation_kinds() -> HashSet<ObligationKind> {
        HashSet::from([
            ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::AliPay),
        ])
    }

    pub fn maker_obligation_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 1000000, // 1M RMB
            amount_min: None,
        }
    }

    pub fn taker_obligation_kinds() -> HashSet<ObligationKind> {
        HashSet::from([ObligationKind::Bitcoin(BitcoinSettlementMethod::Lightning)])
    }

    pub fn taker_obligation_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(40.0), // 10,000,000 Sats/ 250,000 RMB (@ ~$35k USD / BTC) = 40.00
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn trade_parameters() -> HashSet<TradeParameter> {
        HashSet::from([
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::FourDays),
        ])
    }

    pub fn trade_details_content() -> TradeDetailsContent {
        TradeDetailsContent {
            maker_bond_pct: Some(10),
            taker_bond_pct: Some(10),
            trade_timeout: None,
        }
    }

    pub fn pow_difficulty() -> u64 {
        8u64
    }

    pub fn expected_json_string() -> String {
        "{\"maker_obligation\":{\"amount\":1000000,\"amount_min\":null},\"taker_obligation\":{\"limit_rate\":40.0,\"market_offset_pct\":null,\"market_oracles\":null},\"trade_details\":{\"maker_bond_pct\":10,\"taker_bond_pct\":10,\"trade_timeout\":null},\"trade_engine_specifics\":{\"type\":\"some-trade-engine-maker-order-specifics\",\"test_specific_field\":\"some-test-specific-info\"},\"pow_difficulty\":8}".to_string()
    }

    pub fn default_builder() -> OrderBuilder {
        let mut builder: OrderBuilder = OrderBuilder::new();
        builder.trade_uuid(Self::some_uuid());

        let maker_obligation = MakerObligation {
            kinds: Self::maker_obligation_kinds(),
            content: Self::maker_obligation_content(),
        };

        builder.maker_obligation(maker_obligation);

        let taker_obligation = TakerObligation {
            kinds: Self::taker_obligation_kinds(),
            content: Self::taker_obligation_content(),
        };

        builder.taker_obligation(taker_obligation);

        let trade_details = TradeDetails {
            parameters: Self::trade_parameters(),
            content: Self::trade_details_content(),
        };

        builder.trade_details(trade_details);

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(Self::pow_difficulty());

        builder
    }

    pub fn check(order: &Order, expected: &Order) {
        assert_eq!(order.trade_uuid, expected.trade_uuid);
        assert_eq!(
            order.maker_obligation.kinds,
            expected.maker_obligation.kinds
        );
        assert_eq!(
            order.maker_obligation.content.amount,
            expected.maker_obligation.content.amount
        );
        assert_eq!(
            order.maker_obligation.content.amount_min,
            expected.maker_obligation.content.amount_min
        );
        assert_eq!(
            order.taker_obligation.kinds,
            expected.taker_obligation.kinds
        );
        assert_eq!(
            order.taker_obligation.content.limit_rate,
            expected.taker_obligation.content.limit_rate
        );
        assert_eq!(
            order.taker_obligation.content.market_offset_pct,
            expected.taker_obligation.content.market_offset_pct
        );
        assert_eq!(
            order.taker_obligation.content.market_oracles,
            expected.taker_obligation.content.market_oracles
        );
        assert_eq!(
            order.trade_details.parameters,
            expected.trade_details.parameters
        );
        assert_eq!(
            order.trade_details.content.maker_bond_pct,
            expected.trade_details.content.maker_bond_pct
        );
        assert_eq!(
            order.trade_details.content.taker_bond_pct,
            expected.trade_details.content.taker_bond_pct
        );
        assert_eq!(
            order.trade_details.content.trade_timeout,
            expected.trade_details.content.trade_timeout
        );
        let test_trade_engine_specifics = order
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineMakerOrderSpecifics>()
            .unwrap();
        let expected_trade_engine_specifics = expected
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineMakerOrderSpecifics>()
            .unwrap();
        assert_eq!(
            test_trade_engine_specifics.test_specific_field,
            expected_trade_engine_specifics.test_specific_field
        );
        assert_eq!(order.pow_difficulty, expected.pow_difficulty);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SomeTradeEngineMakerOrderSpecifics {
    pub test_specific_field: String,
}

#[typetag::serde(name = "some-trade-engine-maker-order-specifics")]
impl SerdeGenericTrait for SomeTradeEngineMakerOrderSpecifics {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
