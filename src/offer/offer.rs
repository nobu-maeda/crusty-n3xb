use serde::{Deserialize, Serialize};

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
                "Offer Maker Obligation Kind {} not found in initial Order",
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
                if !Self::f64_amount_within_pct_of(order_bond_amount, offer_bond_amount as f64, 0.1)
                {
                    return Err(N3xbError::Simple(format!("Offer Maker Obligation bond amount does not match percentage specified in initial Order")));
                }
            } else {
                return Err(N3xbError::Simple(format!("Offer Maker Obligation does not have bond amount specified as required in the initial Order")));
            }
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
                "Offer Taker Obligation Kind {} not found in initial Order",
                self.taker_obligation.kind
            )));
        }

        let maker_amount = self.maker_obligation.amount as f64; // This is validated in Maker validation. So we take it as it is

        if let Some(limit_rate) = order.taker_obligation.content.limit_rate {
            let expected_taker_amount = maker_amount * limit_rate;
            let taker_amount = self.taker_obligation.amount as f64;
            if !Self::f64_amount_within_pct_of(expected_taker_amount, taker_amount, 0.1) {
                return Err(N3xbError::Simple(format!(
                    "Offer Taker Obligation amount not as expected"
                )));
            }
        }

        if let Some(market_oracle_used) = &self.market_oracle_used {
            if let Some(market_oracles) = &order.taker_obligation.content.market_oracles {
                if !market_oracles.contains(market_oracle_used) {
                    return Err(N3xbError::Simple(format!(
                        "Market Oracle {} not found in list of the initial Order",
                        market_oracle_used
                    )));
                }
            } else {
                return Err(N3xbError::Simple(format!(
                        "Market Oracle {} not expected when intiial Order contains no allowable oracles",
                        market_oracle_used
                    )));
            }
        }

        if order.taker_obligation.content.market_offset_pct.is_some() {
            return Err(N3xbError::Simple(format!(
                "Market & Oracle based rate determination not yet supported "
            )));
        }

        if let Some(taker_bond_pct) = order.trade_details.content.taker_bond_pct {
            let order_bond_amount =
                taker_bond_pct as f64 / 100.0 * self.transacted_sat_amount() as f64;

            // Should be okay to give +/- 0.1% leeway for bond amount
            if let Some(offer_bond_amount) = self.taker_obligation.bond_amount {
                if !Self::f64_amount_within_pct_of(order_bond_amount, offer_bond_amount as f64, 0.1)
                {
                    return Err(N3xbError::Simple(format!("Offer Taker Obligation bond amount does not match percentage specified in initial Order")));
                }
            } else {
                return Err(N3xbError::Simple(format!("Offer Taker Obligation does not have bond amount specified as required in the initial Order")));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        order::{MakerObligation, Order, OrderBuilder, TakerObligation, TradeDetails},
        testing::{
            SomeTestParams, SomeTradeEngineMakerOrderSpecifics, SomeTradeEngineTakerOfferSpecifics,
        },
    };

    use super::Offer;

    fn make_some_order() -> Order {
        // Build and send the Maker Order
        let mut builder: OrderBuilder = OrderBuilder::new();
        builder.pubkey(SomeTestParams::some_x_only_public_key());
        builder.trade_uuid(SomeTestParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestParams::maker_obligation_kinds(),
            content: SomeTestParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestParams::taker_obligation_kinds(),
            content: SomeTestParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestParams::trade_parameters(),
            content: SomeTestParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestParams::pow_difficulty());

        builder.build().unwrap()
    }

    #[tokio::test]
    async fn test_validate_offer() {
        let order = make_some_order();

        let offer = Offer {
            maker_obligation: SomeTestParams::offer_maker_obligation(),
            taker_obligation: SomeTestParams::offer_taker_obligation(),
            market_oracle_used: SomeTestParams::offer_marker_oracle_used(),
            trade_engine_specifics: Box::new(SomeTradeEngineTakerOfferSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            }),
            pow_difficulty: SomeTestParams::offer_pow_difficulty(),
        };

        offer.validate_against(&order).unwrap()
    }

    #[tokio::test]
    async fn test_validate_offer_maker_amount_in_bounds() {}

    #[tokio::test]
    async fn test_validate_offer_bonds_matched() {}

    #[tokio::test]
    async fn test_validate_f64_amount_under() {}

    #[tokio::test]
    async fn test_validate_f64_amount_min() {}

    #[tokio::test]
    async fn test_validate_f64_amount_max() {}

    #[tokio::test]
    async fn test_validate_f64_amount_over() {}

    #[tokio::test]
    async fn test_validate_f64_amount_negative() {}

    #[tokio::test]
    async fn test_validate_f64_amount_overflow() {}

    #[tokio::test]
    async fn test_validate_offer_maker_kind_not_found() {}

    #[tokio::test]
    async fn test_validate_offer_maker_amount_out_of_bounds() {}

    #[tokio::test]
    async fn test_validate_offer_maker_amount_mismatch() {}

    #[tokio::test]
    async fn test_validate_offer_maker_bond_mismatch() {}

    #[tokio::test]
    async fn test_validate_offer_maker_bond_not_found() {}

    #[tokio::test]
    async fn test_validate_offer_maker_bond_not_expected() {}

    #[tokio::test]
    async fn test_validate_offer_taker_kind_not_found() {}

    #[tokio::test]
    async fn test_validate_offer_taker_amount_not_as_expected() {}

    #[tokio::test]
    async fn test_validate_offer_market_oracle_not_found() {}

    #[tokio::test]
    async fn test_validate_offer_market_oracle_not_expected() {}

    #[tokio::test]
    async fn test_validate_offer_taker_bond_mismatch() {}

    #[tokio::test]
    async fn test_validate_offer_taker_bond_not_found() {}

    #[tokio::test]
    async fn test_validate_offer_taker_bond_not_expected() {}
}
