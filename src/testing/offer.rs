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

    // Maker Oligations

    pub fn maker_obligation_rmb_wechat() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::WeChatPay)),
            amount: 35000,
            bond_amount: Some(1000000),
        }
    }

    pub fn maker_obligation_rmb_alipay() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::AliPay)),
            amount: 35000,
            bond_amount: Some(1000000),
        }
    }

    pub fn maker_obligation_usd_zelle() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::Zelle)),
            amount: 5000,
            bond_amount: Some(1000000),
        }
    }

    pub fn maker_obligation_eur_revolut() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::EUR, Some(FiatPaymentMethod::Revolut)),
            amount: 4500,
            bond_amount: Some(1000000),
        }
    }

    pub fn maker_obligation_bitcoin_onchain() -> Obligation {
        Obligation {
            kind: ObligationKind::Bitcoin(Some(BitcoinSettlementMethod::Onchain)),
            amount: 10000000,
            bond_amount: Some(1000000),
        }
    }

    pub fn maker_obligation_bitcoin_lightning() -> Obligation {
        Obligation {
            kind: ObligationKind::Bitcoin(Some(BitcoinSettlementMethod::Lightning)),
            amount: 10000000,
            bond_amount: Some(1000000),
        }
    }

    // Taker Obligation

    pub fn taker_obligation_rmb_wechat() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::WeChatPay)),
            amount: 35000,
            bond_amount: Some(1000000),
        }
    }

    pub fn taker_obligation_rmb_alipay() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::AliPay)),
            amount: 35000,
            bond_amount: Some(1000000),
        }
    }

    pub fn taker_obligation_usd_zelle() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::USD, Some(FiatPaymentMethod::Zelle)),
            amount: 5000,
            bond_amount: Some(1000000),
        }
    }

    pub fn taker_obligation_eur_revolut() -> Obligation {
        Obligation {
            kind: ObligationKind::Fiat(Currency::EUR, Some(FiatPaymentMethod::Revolut)),
            amount: 4500,
            bond_amount: Some(1000000),
        }
    }

    pub fn taker_obligation_bitcoin_onchain() -> Obligation {
        Obligation {
            kind: ObligationKind::Bitcoin(Some(BitcoinSettlementMethod::Onchain)),
            amount: 10000000,
            bond_amount: Some(1000000),
        }
    }

    pub fn taker_obligation_bitcoin_lightning() -> Obligation {
        Obligation {
            kind: ObligationKind::Bitcoin(Some(BitcoinSettlementMethod::Lightning)),
            amount: 10000000,
            bond_amount: Some(1000000),
        }
    }

    pub fn default_builder() -> OfferBuilder {
        let mut builder: OfferBuilder = OfferBuilder::new();
        builder.maker_obligation(Self::maker_obligation_rmb_wechat());
        builder.taker_obligation(Self::taker_obligation_bitcoin_lightning());

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
