use std::any::Any;
use std::collections::HashSet;
use std::str::FromStr;

use iso_currency::Currency;
use secp256k1::{PublicKey, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use url::Url;
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

    // Obligation Kinds

    pub fn obligation_fiat_cny_kinds() -> HashSet<ObligationKind> {
        HashSet::from([
            ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::WeChatPay)),
            ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::AliPay)),
        ])
    }

    pub fn obligation_fiat_usd_kinds() -> HashSet<ObligationKind> {
        HashSet::from([
            ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::Zelle)),
            ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::ACHTransfer)),
        ])
    }

    pub fn obligation_fiat_eur_kinds() -> HashSet<ObligationKind> {
        HashSet::from([ObligationKind::Fiat(
            Currency::EUR,
            Some(FiatPaymentMethod::Revolut),
        )])
    }

    pub fn obligation_bitcoin_onchain_kinds() -> HashSet<ObligationKind> {
        HashSet::from([ObligationKind::Bitcoin(
            BitcoinNetwork::Testnet,
            Some(BitcoinSettlementMethod::Onchain),
        )])
    }

    pub fn obligation_bitcoin_lightning_kinds() -> HashSet<ObligationKind> {
        HashSet::from([ObligationKind::Bitcoin(
            BitcoinNetwork::Testnet,
            Some(BitcoinSettlementMethod::Lightning),
        )])
    }

    pub fn obligation_bitcoin_both_kinds() -> HashSet<ObligationKind> {
        HashSet::from([
            ObligationKind::Bitcoin(
                BitcoinNetwork::Testnet,
                Some(BitcoinSettlementMethod::Onchain),
            ),
            ObligationKind::Bitcoin(
                BitcoinNetwork::Testnet,
                Some(BitcoinSettlementMethod::Lightning),
            ),
        ])
    }

    // Maker Obligation Contents

    pub fn maker_obligation_fiat_cny_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 35000.0, // 35k RMB
            amount_min: None,
        }
    }

    pub fn maker_obligation_fiat_usd_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 5000.0, // 5k USD
            amount_min: Some(3000.0),
        }
    }

    pub fn maker_obligation_fiat_eur_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 4500.0, // 4.5k EUR
            amount_min: None,
        }
    }

    pub fn maker_obligation_bitcoin_content() -> MakerObligationContent {
        MakerObligationContent {
            amount: 10000000.0, // 10,000,000 Sats / 0.1 BTC
            amount_min: None,
        }
    }

    // Taker Obligation Contents

    pub fn taker_obligation_fiat_cny_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(0.0035), // 35,000 RMB / 10,000,000 Sats (@ ~$50k USD / BTC) = 0.0035
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn taker_obligation_fiat_usd_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(0.0005), // 5,000 USD / 10,000,000 Sats (@ ~$50k USD / BTC) = 0.0005
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn taker_obligation_fiat_eur_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(0.00045), // 4,500 EUR / 10,000,000 Sats (@ ~$50k USD / BTC) = 0.00045
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn taker_obligation_bitcoin_rmb_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(285.71429), // 10,000,000 Sats/ 35,000 RMB (@ ~$50k USD / BTC) = 285.71
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn taker_obligation_bitcoin_usd_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(2000.0), // 10,000,000 Sats/ 5,000 USD (@ $50k USD / BTC) = 2000
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    pub fn taker_obligation_bitcoin_eur_content() -> TakerObligationContent {
        TakerObligationContent {
            limit_rate: Some(2222.22222), // 10,000,000 Sats/ 4,500 EUR (@ ~$50k USD / BTC) = 2222.22
            market_offset_pct: None,
            market_oracles: None,
        }
    }

    // Trade Parameters

    pub fn trade_parameters() -> HashSet<TradeParameter> {
        HashSet::from([
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::FourDays),
        ])
    }

    pub fn trade_parameters_empty() -> HashSet<TradeParameter> {
        HashSet::from([])
    }

    pub fn trade_details_content() -> TradeDetailsContent {
        TradeDetailsContent {
            maker_bond_pct: Some(10),
            taker_bond_pct: Some(10),
            trade_timeout: None,
        }
    }

    pub fn trade_details_empty() -> TradeDetailsContent {
        TradeDetailsContent {
            maker_bond_pct: None,
            taker_bond_pct: None,
            trade_timeout: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn event_kind() -> EventKind {
        EventKind::MakerOrder
    }

    pub fn application_tag() -> String {
        "n3xB".to_string()
    }

    pub fn pow_difficulty() -> u64 {
        8u64
    }

    pub fn expected_json_string() -> String {
        "{\"maker_obligation\":{\"amount\":35000,\"amount_min\":null},\"taker_obligation\":{\"limit_rate\":285.71,\"market_offset_pct\":null,\"market_oracles\":null},\"trade_details\":{\"maker_bond_pct\":10,\"taker_bond_pct\":10,\"trade_timeout\":null},\"trade_engine_specifics\":{\"type\":\"some-trade-engine-maker-order-specifics\",\"test_specific_field\":\"some-test-specific-info\"},\"pow_difficulty\":8}".to_string()
    }

    pub fn default_buy_builder() -> OrderBuilder {
        let mut builder: OrderBuilder = OrderBuilder::new();
        builder.trade_uuid(Self::some_uuid());

        let maker_obligation = MakerObligation {
            kinds: Self::obligation_fiat_cny_kinds(),
            content: Self::maker_obligation_fiat_cny_content(),
        };

        builder.maker_obligation(maker_obligation);

        let taker_obligation = TakerObligation {
            kinds: Self::obligation_bitcoin_lightning_kinds(),
            content: Self::taker_obligation_bitcoin_rmb_content(),
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

    pub fn default_sell_builder() -> OrderBuilder {
        let mut builder: OrderBuilder = OrderBuilder::new();
        builder.trade_uuid(Self::some_uuid());

        let maker_obligation = MakerObligation {
            kinds: Self::obligation_bitcoin_both_kinds(),
            content: Self::maker_obligation_bitcoin_content(),
        };

        builder.maker_obligation(maker_obligation);

        let taker_obligation = TakerObligation {
            kinds: Self::obligation_fiat_eur_kinds(),
            content: Self::taker_obligation_fiat_eur_content(),
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

    pub fn filter_for_trade_uuid(
        trade_uuid: Uuid,
        order_envelopes: Vec<OrderEnvelope>,
    ) -> Vec<OrderEnvelope> {
        order_envelopes
            .into_iter()
            .filter(|order_envelope| order_envelope.order.trade_uuid == trade_uuid)
            .collect()
    }

    pub fn filter_for_relay_url(
        relay_url: Url,
        order_envelopes: Vec<OrderEnvelope>,
    ) -> Vec<OrderEnvelope> {
        order_envelopes
            .into_iter()
            .filter(|order_envelope| order_envelope.urls.contains(&relay_url))
            .collect()
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
