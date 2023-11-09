use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::{any::Any, fmt::Debug};

use crate::{
    common::{error::N3xbError, types::*},
    order::Order,
};

// Take Order Message Data Structure

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obligation {
    pub kind: ObligationKind,
    pub amount: u64,
    pub bond_amount: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offer {
    pub offer_uuid: Uuid,
    pub maker_obligation: Obligation,
    pub taker_obligation: Obligation,
    pub market_oracle_used: Option<String>, // TODO: Change to URL type
    pub trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    pub pow_difficulty: Option<u64>,
}

#[typetag::serde(name = "n3xB-taker-offer")]
impl SerdeGenericTrait for Offer {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl Offer {
    pub fn validate_against(&self, order: &Order) -> Result<(), N3xbError> {
        self.validate_maker_obligation_against(order)?;
        self.validate_taker_obligation_against(order)?;

        // Check Taker suggested PoW difficulty is higher than in initial Maker Order
        if let Some(pow_difficulty) = self.pow_difficulty {
            if pow_difficulty < order.pow_difficulty {
                return Err(N3xbError::Simple(format!(
                    "Taker Offer suggested lower PoW difficulty than specified in initial Order"
                )));
            }
        }

        // TODO: How to validate trade engine specifics? Depend on the Trade Engine to do so after it gets notified?
        Ok(())
    }

    fn f64_amount_within_pct_of(float1: f64, float2: f64, pct: f64) -> bool {
        let max = float1 * (1.0 + pct / 100.0);
        let min = float1 * (1.0 - pct / 100.0);
        return min <= float2 && float2 <= max;
    }

    fn transacted_sat_amount(&self) -> u64 {
        return if self.maker_obligation.kind.is_bitcoin() {
            self.maker_obligation.amount
        } else if self.taker_obligation.kind.is_bitcoin() {
            self.taker_obligation.amount
        } else {
            panic!("Neither Maker nor Taker has Bitcoin obligation in Offer")
        };
    }

    fn validate_maker_obligation_against(&self, order: &Order) -> Result<(), N3xbError> {
        if !order
            .maker_obligation
            .kinds
            .contains(&self.maker_obligation.kind)
        {
            return Err(N3xbError::Simple(format!(
                "Offer Maker Obligation Kind {:?} not found in initial Order",
                self.maker_obligation.kind
            )));
        }

        if let Some(amount_min) = order.maker_obligation.content.amount_min {
            if self.maker_obligation.amount < amount_min
                || self.maker_obligation.amount > order.maker_obligation.content.amount
            {
                return Err(N3xbError::Simple(format!(
                    "Offer Maker Obligation amount not within bounds specificed in initial Order"
                )));
            }
        } else if self.maker_obligation.amount != order.maker_obligation.content.amount {
            return Err(N3xbError::Simple(format!(
                "Offer Maker Obligation amount does not match amount specified in initial Order"
            )));
        }

        if let Some(maker_bond_pct) = order.trade_details.content.maker_bond_pct {
            let order_bond_amount =
                maker_bond_pct as f64 / 100.0 * self.transacted_sat_amount() as f64;

            // Should be okay to give +/- 0.1% leeway for bond amount
            if let Some(offer_bond_amount) = self.maker_obligation.bond_amount {
                if !Self::f64_amount_within_pct_of(
                    order_bond_amount,
                    offer_bond_amount as f64,
                    0.001,
                ) {
                    return Err(N3xbError::Simple(format!("Offer Maker Obligation bond amount does not match percentage specified in initial Order")));
                }
            } else {
                return Err(N3xbError::Simple(format!("Offer Maker Obligation does not have bond amount specified as required in the initial Order")));
            }
        } else if self.maker_obligation.bond_amount != None {
            return Err(N3xbError::Simple(format!("Offer Maker Obligation should not have bond amount as its not specified in initial Order")));
        }

        Ok(())
    }

    fn validate_taker_obligation_against(&self, order: &Order) -> Result<(), N3xbError> {
        if !order
            .taker_obligation
            .kinds
            .contains(&self.taker_obligation.kind)
        {
            return Err(N3xbError::Simple(format!(
                "Offer Taker Obligation Kind {:?} not found in initial Order",
                self.taker_obligation.kind
            )));
        }

        let maker_amount = self.maker_obligation.amount as f64; // This is validated in Maker validation. So we take it as it is

        if let Some(limit_rate) = order.taker_obligation.content.limit_rate {
            let expected_taker_amount = maker_amount * limit_rate;
            let taker_amount = self.taker_obligation.amount as f64;
            if !Self::f64_amount_within_pct_of(expected_taker_amount, taker_amount, 0.001) {
                return Err(N3xbError::Simple(format!(
                    "Offer Taker Obligation amount not as expected"
                )));
            }
        }

        if self.market_oracle_used.is_some() {
            return Err(N3xbError::Simple(format!(
                "Market & Oracle based rate determination not yet supported"
            )));
        }

        if let Some(taker_bond_pct) = order.trade_details.content.taker_bond_pct {
            let order_bond_amount =
                taker_bond_pct as f64 / 100.0 * self.transacted_sat_amount() as f64;

            // Should be okay to give +/- 0.1% leeway for bond amount
            if let Some(offer_bond_amount) = self.taker_obligation.bond_amount {
                if !Self::f64_amount_within_pct_of(
                    order_bond_amount,
                    offer_bond_amount as f64,
                    0.001,
                ) {
                    return Err(N3xbError::Simple(format!("Offer Taker Obligation bond amount does not match percentage specified in initial Order")));
                }
            } else {
                return Err(N3xbError::Simple(format!("Offer Taker Obligation does not have bond amount specified as required in the initial Order")));
            }
        } else if self.taker_obligation.bond_amount != None {
            return Err(N3xbError::Simple(format!("Offer Taker Obligation should not have bond amount as its not specified in initial Order")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use iso_currency::Currency;

    use crate::{
        common::types::{BitcoinSettlementMethod, FiatPaymentMethod, ObligationKind},
        offer::Obligation,
        order::{MakerObligation, MakerObligationContent, TradeDetails, TradeDetailsContent},
        testing::{SomeTestOfferParams, SomeTestOrderParams},
    };

    #[tokio::test]
    async fn test_validate_offer() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();
        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        offer.validate_against(&order).unwrap();
    }

    #[tokio::test]
    async fn test_validate_offer_maker_amount_in_bounds() {
        let maker_obligation_content = MakerObligationContent {
            amount: 1200000,
            amount_min: Some(800000),
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.maker_obligation(maker_obligation).build().unwrap();
        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        offer.validate_against(&order).unwrap();
    }

    #[tokio::test]
    async fn test_validate_offer_maker_kind_not_found() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let maker_obligation = Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::FaceToFace),
            amount: 1000000,
            bond_amount: Some(4000000),
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.maker_obligation(maker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_maker_f64_amount_under() {
        let maker_obligation_content = MakerObligationContent {
            amount: 2000000,
            amount_min: Some(1200000),
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.maker_obligation(maker_obligation).build().unwrap();
        let offer = SomeTestOfferParams::default_builder().build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_maker_f64_amount_min() {
        let maker_obligation_content = MakerObligationContent {
            amount: 2000000,
            amount_min: Some(1000000),
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.maker_obligation(maker_obligation).build().unwrap();

        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        offer.validate_against(&order).unwrap();
    }

    #[tokio::test]
    async fn test_validate_offer_maker_f64_amount_max() {
        let maker_obligation_content = MakerObligationContent {
            amount: 1000000,
            amount_min: Some(700000),
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.maker_obligation(maker_obligation).build().unwrap();

        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        offer.validate_against(&order).unwrap();
    }

    #[tokio::test]
    async fn test_validate_offer_maker_f64_amount_over() {
        let maker_obligation_content = MakerObligationContent {
            amount: 800000,
            amount_min: Some(500000),
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.maker_obligation(maker_obligation).build().unwrap();

        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_maker_f64_amount_overflow() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let maker_obligation = Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            amount: u64::MAX,
            bond_amount: Some(4000000),
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.maker_obligation(maker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_maker_bond_mismatch() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let maker_obligation = Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            amount: 1000000,
            bond_amount: Some(3000000),
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.maker_obligation(maker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_maker_bond_not_found() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let maker_obligation = Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            amount: 1000000,
            bond_amount: None,
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.maker_obligation(maker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_maker_bond_not_expected() {
        let trade_details = TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: TradeDetailsContent {
                maker_bond_pct: None,
                taker_bond_pct: Some(10),
                trade_timeout: None,
            },
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.trade_details(trade_details).build().unwrap();
        let offer = SomeTestOfferParams::default_builder().build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_taker_kind_not_found() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let taker_obligation = Obligation {
            kind: ObligationKind::Bitcoin(BitcoinSettlementMethod::Onchain),
            amount: 40000000,
            bond_amount: Some(4000000),
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.taker_obligation(taker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_taker_amount_not_as_expected() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let maker_obligation = Obligation {
            kind: ObligationKind::Fiat(Currency::CNY, FiatPaymentMethod::WeChatPay),
            amount: 1000000,
            bond_amount: Some(4200000),
        };

        let taker_obligation = Obligation {
            kind: ObligationKind::Bitcoin(BitcoinSettlementMethod::Lightning),
            amount: 42000000,
            bond_amount: Some(4200000),
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.maker_obligation(maker_obligation);
        builder.taker_obligation(taker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_taker_bond_mismatch() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let taker_obligation = Obligation {
            kind: ObligationKind::Bitcoin(BitcoinSettlementMethod::Lightning),
            amount: 40000000,
            bond_amount: Some(3000000),
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.taker_obligation(taker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_taker_bond_not_found() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let taker_obligation = Obligation {
            kind: ObligationKind::Bitcoin(BitcoinSettlementMethod::Lightning),
            amount: 40000000,
            bond_amount: None,
        };

        let mut builder = SomeTestOfferParams::default_builder();
        builder.taker_obligation(taker_obligation);
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_taker_bond_not_expected() {
        let trade_details = TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: TradeDetailsContent {
                maker_bond_pct: Some(10),
                taker_bond_pct: None,
                trade_timeout: None,
            },
        };

        let mut builder = SomeTestOrderParams::default_builder();
        let order = builder.trade_details(trade_details).build().unwrap();
        let offer = SomeTestOfferParams::default_builder().build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_offer_market_oracle_not_yet_supported() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();

        let mut builder = SomeTestOfferParams::default_builder();
        builder.market_oracle_used("https://www.bitstamp.com/api/".to_string());
        let offer = builder.build().unwrap();

        let result = offer.validate_against(&order);
        assert!(result.is_err());
    }
}
