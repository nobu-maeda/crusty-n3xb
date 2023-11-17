use std::collections::HashSet;

use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::{obligation::*, trade_details::*};
use crate::{
    common::{
        error::N3xbError,
        types::{EventIdString, ObligationKind, SerdeGenericTrait},
    },
    offer::OfferBuilder,
};

#[derive(Clone, Debug)]
pub struct OrderEnvelope {
    pub pubkey: XOnlyPublicKey,
    pub urls: HashSet<Url>,
    pub event_id: EventIdString,
    pub order: Order,
    pub(crate) _private: (),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub trade_uuid: Uuid,
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    pub pow_difficulty: u64,
    pub(crate) _private: (),
}

impl Order {
    pub fn validate(&self) -> Result<(), N3xbError> {
        // Add additional validation rules here. Code is doc in this case
        self.validate_maker_obligation_kinds_has_settlement()?;
        self.validate_maker_obligation_kinds_currencies_same()?;
        self.validate_maker_obligation_amount_valid()?;
        self.validate_taker_obligation_kinds_has_settlement()?;
        self.validate_taker_obligation_kinds_currencies_same()?;
        self.validate_taker_obligation_specified()?;
        self.validate_taker_obligation_limit_rate_valid()?;
        self.validate_taker_obligation_market_offset_not_supported()?;
        self.validate_trade_details_bonds_required()?;
        Ok(())
    }

    fn validate_maker_obligation_kinds_has_settlement(&self) -> Result<(), N3xbError> {
        for maker_obligation_kind in &self.maker_obligation.kinds {
            match maker_obligation_kind {
                ObligationKind::Fiat(_currency, method) => {
                    if method.is_none() {
                        return Err(N3xbError::Simple(format!(
                            "Maker Obligation Kinds in Order missing Settlement Method"
                        )));
                    }
                }
                ObligationKind::Bitcoin(method) => {
                    if method.is_none() {
                        return Err(N3xbError::Simple(format!(
                            "Maker Obligation Kinds in Order missing Settlement Method"
                        )));
                    }
                }
                ObligationKind::Custom(_) => {}
            }
        }
        Ok(())
    }

    fn validate_maker_obligation_kinds_currencies_same(&self) -> Result<(), N3xbError> {
        let mut obligation_kind: Option<ObligationKind> = None;

        for maker_obligation_kind in &self.maker_obligation.kinds {
            if let Some(kind) = obligation_kind.to_owned() {
                if !maker_obligation_kind.is_same_currency_as(kind) {
                    return Err(N3xbError::Simple(format!(
                        "Maker Obligation Kinds in Order not all of the same kinds"
                    )));
                }
            } else {
                obligation_kind = Some(maker_obligation_kind.clone());
            }
        }
        Ok(())
    }

    fn validate_maker_obligation_amount_valid(&self) -> Result<(), N3xbError> {
        if self.maker_obligation.content.amount == 0 {
            return Err(N3xbError::Simple(format!(
                "Maker Obligation Kind amount should not be zero"
            )));
        } else if let Some(min) = self.maker_obligation.content.amount_min {
            if min > self.maker_obligation.content.amount {
                return Err(N3xbError::Simple(format!(
                    "Maker Obligation amount less than minimum"
                )));
            }
        }
        Ok(())
    }

    fn validate_taker_obligation_kinds_has_settlement(&self) -> Result<(), N3xbError> {
        for taker_obligation_kind in &self.taker_obligation.kinds {
            match taker_obligation_kind {
                ObligationKind::Fiat(_currency, method) => {
                    if method.is_none() {
                        return Err(N3xbError::Simple(format!(
                            "Taker Obligation Kinds in Order missing Settlement Method"
                        )));
                    }
                }
                ObligationKind::Bitcoin(method) => {
                    if method.is_none() {
                        return Err(N3xbError::Simple(format!(
                            "Taker Obligation Kinds in Order missing Settlement Method"
                        )));
                    }
                }
                ObligationKind::Custom(_) => {}
            }
        }
        Ok(())
    }

    fn validate_taker_obligation_kinds_currencies_same(&self) -> Result<(), N3xbError> {
        let mut obligation_kind: Option<ObligationKind> = None;

        for taker_obligation_kind in &self.taker_obligation.kinds {
            if let Some(kind) = obligation_kind.to_owned() {
                if !taker_obligation_kind.is_same_currency_as(kind) {
                    return Err(N3xbError::Simple(format!(
                        "Taker Obligation Kinds in Order not all of the same kinds"
                    )));
                }
            } else {
                obligation_kind = Some(taker_obligation_kind.clone());
            }
        }
        Ok(())
    }

    fn validate_taker_obligation_specified(&self) -> Result<(), N3xbError> {
        if self.taker_obligation.content.limit_rate.is_none()
            && (self.taker_obligation.content.market_offset_pct.is_none()
                || self.taker_obligation.content.market_oracles.is_none())
        {
            return Err(N3xbError::Simple(format!(
                "Taker Obligation does not have Limit Rate nor Market Offset specified"
            )));
        }
        Ok(())
    }

    fn validate_taker_obligation_limit_rate_valid(&self) -> Result<(), N3xbError> {
        if let Some(limit_rate) = self.taker_obligation.content.limit_rate {
            if limit_rate <= 0.0 {
                return Err(N3xbError::Simple(format!(
                    "Taker Obligation Limit Rate cannot be zero or lower"
                )));
            }
        }
        Ok(())
    }

    fn validate_taker_obligation_market_offset_not_supported(&self) -> Result<(), N3xbError> {
        if self.taker_obligation.content.market_offset_pct.is_some()
            || self.taker_obligation.content.market_oracles.is_some()
        {
            return Err(N3xbError::Simple(format!(
                "Taker Obligation market offset and oracle not yet supported"
            )));
        }
        Ok(())
    }

    fn validate_trade_details_bonds_required(&self) -> Result<(), N3xbError> {
        if self
            .trade_details
            .parameters
            .contains(&TradeParameter::BondsRequired)
        {
            if self.trade_details.content.maker_bond_pct.is_none() {
                return Err(N3xbError::Simple(format!(
                    "Trade Details Maker Bond Percentage should be specified"
                )));
            }

            if self.trade_details.content.taker_bond_pct.is_none() {
                return Err(N3xbError::Simple(format!(
                    "Trade Details Taker Bond Percentage should be specified"
                )));
            }

            if let Some(maker_bond_pct) = self.trade_details.content.maker_bond_pct {
                if let Some(taker_bond_pct) = self.trade_details.content.taker_bond_pct {
                    if maker_bond_pct == 0 && taker_bond_pct == 0 {
                        return Err(N3xbError::Simple(format!(
                            "Trade Details Maker & Taker Bond Percentages should not be both zeros"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        common::types::{FiatPaymentMethod, ObligationKind},
        order::{
            MakerObligation, MakerObligationContent, TakerObligation, TakerObligationContent,
            TradeDetails, TradeDetailsContent, TradeParameter, TradeTimeOutLimit,
        },
        testing::SomeTestOrderParams,
    };

    use iso_currency::Currency;

    #[tokio::test]
    async fn test_validate_order() {
        _ = SomeTestOrderParams::default_builder().build().unwrap();
    }

    #[tokio::test]
    async fn test_validate_order_maker_obligation_kind_fiat_missing_settlement() {
        let maker_obligation_kinds = HashSet::from([
            ObligationKind::Fiat(Currency::USD, None),
            ObligationKind::Fiat(Currency::EUR, Some(FiatPaymentMethod::ACHTransfer)),
        ]);
        let maker_obligation = MakerObligation {
            kinds: maker_obligation_kinds,
            content: SomeTestOrderParams::maker_obligation_content(),
        };

        let result = SomeTestOrderParams::default_builder()
            .maker_obligation(maker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_maker_obligation_kind_bitcoin_missing_settlement() {
        let maker_obligation_kinds = HashSet::from([ObligationKind::Bitcoin(None)]);
        let maker_obligation = MakerObligation {
            kinds: maker_obligation_kinds,
            content: SomeTestOrderParams::maker_obligation_content(),
        };

        let result = SomeTestOrderParams::default_builder()
            .maker_obligation(maker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_maker_obligation_kind_currencies_mismatches() {
        let maker_obligation_kinds = HashSet::from([
            ObligationKind::Fiat(Currency::JPY, Some(FiatPaymentMethod::TransferWise)),
            ObligationKind::Fiat(Currency::EUR, Some(FiatPaymentMethod::TransferWise)),
        ]);

        let maker_obligation = MakerObligation {
            kinds: maker_obligation_kinds,
            content: SomeTestOrderParams::maker_obligation_content(),
        };

        let result = SomeTestOrderParams::default_builder()
            .maker_obligation(maker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_maker_obligation_amount_zero() {
        let maker_obligation_content = MakerObligationContent {
            amount: 0,
            amount_min: None,
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .maker_obligation(maker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_maker_obligation_amount_less_than_min() {
        let maker_obligation_content = MakerObligationContent {
            amount: 1000000,
            amount_min: Some(1000001),
        };

        let maker_obligation = MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: maker_obligation_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .maker_obligation(maker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_kind_fiat_missing_settlement() {
        let taker_obligation_kinds = HashSet::from([
            ObligationKind::Fiat(Currency::USD, None),
            ObligationKind::Fiat(Currency::EUR, Some(FiatPaymentMethod::ACHTransfer)),
        ]);
        let taker_obligation = TakerObligation {
            kinds: taker_obligation_kinds,
            content: SomeTestOrderParams::taker_obligation_content(),
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_kind_bitcoin_missing_settlement() {
        let taker_obligation_kinds = HashSet::from([ObligationKind::Bitcoin(None)]);
        let taker_obligation = TakerObligation {
            kinds: taker_obligation_kinds,
            content: SomeTestOrderParams::taker_obligation_content(),
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_kind_currencies_mismatches() {
        let taker_obligation_kinds = HashSet::from([
            ObligationKind::Fiat(Currency::TWD, Some(FiatPaymentMethod::TransferWise)),
            ObligationKind::Fiat(Currency::CNY, Some(FiatPaymentMethod::TransferWise)),
        ]);

        let taker_obligation = TakerObligation {
            kinds: taker_obligation_kinds,
            content: SomeTestOrderParams::taker_obligation_content(),
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_under_specified() {
        let taker_obligation_content = TakerObligationContent {
            limit_rate: None,
            market_offset_pct: None,
            market_oracles: None,
        };

        let taker_obligation = TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: taker_obligation_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_limit_rate_zero() {
        let taker_obligation_content = TakerObligationContent {
            limit_rate: Some(0.0),
            market_offset_pct: None,
            market_oracles: None,
        };

        let taker_obligation = TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: taker_obligation_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_limit_rate_negative() {
        let taker_obligation_content = TakerObligationContent {
            limit_rate: Some(-40.0),
            market_offset_pct: None,
            market_oracles: None,
        };

        let taker_obligation = TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: taker_obligation_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_obligation_market_offset() {
        let market_oracles = HashSet::from([
            "https://www.bitstamp.com/api/".to_string(),
            "https://www.kraken.com/api/".to_string(),
        ]);
        let taker_obligation_content = TakerObligationContent {
            limit_rate: None,
            market_offset_pct: Some(1.0),
            market_oracles: Some(market_oracles),
        };

        let taker_obligation = TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: taker_obligation_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .taker_obligation(taker_obligation)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_maker_bond_pct_missing() {
        let trade_parameters = HashSet::from([
            TradeParameter::BondsRequired,
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::FourDays),
        ]);

        let trade_details_content = TradeDetailsContent {
            maker_bond_pct: None,
            taker_bond_pct: Some(10),
            trade_timeout: None,
        };

        let trade_details = TradeDetails {
            parameters: trade_parameters,
            content: trade_details_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .trade_details(trade_details)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_taker_bond_pct_missing() {
        let trade_parameters = HashSet::from([
            TradeParameter::BondsRequired,
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::OneDay),
        ]);

        let trade_details_content = TradeDetailsContent {
            maker_bond_pct: Some(10),
            taker_bond_pct: None,
            trade_timeout: None,
        };

        let trade_details = TradeDetails {
            parameters: trade_parameters,
            content: trade_details_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .trade_details(trade_details)
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_order_bond_pcts_both_zeros() {
        let trade_parameters = HashSet::from([
            TradeParameter::BondsRequired,
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::FourDays),
        ]);

        let trade_details_content = TradeDetailsContent {
            maker_bond_pct: Some(0),
            taker_bond_pct: Some(0),
            trade_timeout: None,
        };

        let trade_details = TradeDetails {
            parameters: trade_parameters,
            content: trade_details_content,
        };

        let result = SomeTestOrderParams::default_builder()
            .trade_details(trade_details)
            .build();
        assert!(result.is_err());
    }
}
