use std::any::Any;
use std::str::FromStr;

use iso_currency::Currency;
use secp256k1::{PublicKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

use crate::common::types::*;
use crate::offer::*;

use super::SomeTestParams;

pub struct SomeTestOfferParams {}

impl SomeTestOfferParams {
    pub fn some_x_only_public_key() -> XOnlyPublicKey {
        let kpk = PublicKey::from_str(
            "0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd166",
        )
        .unwrap();
        XOnlyPublicKey::from(kpk)
    }

    pub fn maker_obligation() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            amount: 1000000,
            bond_amount: Some(4000000),
        }
    }

    pub fn taker_obligation() -> Obligation {
        Obligation {
            kind: ObligationKind::Bitcoin(BitcoinSettlementMethod::Lightning),
            amount: 40000000,
            bond_amount: Some(4000000),
        }
    }

    pub fn default_builder() -> OfferBuilder {
        let mut builder: OfferBuilder = OfferBuilder::new();
        builder.maker_obligation(Self::maker_obligation());
        builder.taker_obligation(Self::taker_obligation());

        let trade_engine_specifics = Box::new(SomeTradeEngineTakerOfferSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);
        builder
    }

    pub fn check(offer: &Offer, expected: &Offer) {
        assert_eq!(offer.maker_obligation.kind, expected.maker_obligation.kind);
        assert_eq!(
            offer.maker_obligation.amount,
            expected.maker_obligation.amount
        );
        assert_eq!(
            offer.maker_obligation.bond_amount,
            expected.maker_obligation.bond_amount
        );
        assert_eq!(offer.taker_obligation.kind, expected.taker_obligation.kind);
        assert_eq!(
            offer.taker_obligation.amount,
            expected.taker_obligation.amount
        );
        assert_eq!(
            offer.taker_obligation.bond_amount,
            expected.taker_obligation.bond_amount
        );
        let test_trade_engine_specifics = offer
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineTakerOfferSpecifics>()
            .unwrap();
        let expected_test_trade_engine_specifics = expected
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineTakerOfferSpecifics>()
            .unwrap();
        assert_eq!(
            test_trade_engine_specifics.test_specific_field,
            expected_test_trade_engine_specifics.test_specific_field
        );
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SomeTradeEngineTakerOfferSpecifics {
    pub test_specific_field: String,
}

#[typetag::serde(name = "some-trade-engine-taker-offer-specifics")]
impl SerdeGenericTrait for SomeTradeEngineTakerOfferSpecifics {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
