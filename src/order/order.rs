use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{obligation::*, trade_details::*};
use crate::common::{
    error::N3xbError,
    types::{ObligationKind, SerdeGenericTrait},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub pubkey: XOnlyPublicKey,
    pub event_id: String,
    pub trade_uuid: Uuid,
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    pub pow_difficulty: u64,
}

impl Order {
    pub fn validate(&self) -> Result<(), N3xbError> {
        self.validate_maker_obligation_kinds_same()?;
        self.validate_maker_obligation_amount_valid()?;
        self.validate_taker_obligation_kinds_same()?;
        self.validate_taker_obligation_specified()?;
        self.validate_taker_obligation_limit_rate_valid()?;
        self.validate_taker_obligation_market_offset_not_supported()?;
        self.validate_trade_details_bonds_required()?;
        Ok(())
    }

    fn validate_maker_obligation_kinds_same(&self) -> Result<(), N3xbError> {
        let mut obligation_kind: Option<ObligationKind> = None;

        for maker_obligation_kind in &self.maker_obligation.kinds {
            if let Some(kind) = obligation_kind.to_owned() {
                if kind != maker_obligation_kind.to_owned() {
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

    fn validate_taker_obligation_kinds_same(&self) -> Result<(), N3xbError> {
        let mut obligation_kind: Option<ObligationKind> = None;

        for taker_obligation_kind in &self.taker_obligation.kinds {
            if let Some(kind) = obligation_kind.to_owned() {
                if kind != taker_obligation_kind.to_owned() {
                    return Err(N3xbError::Simple(format!(
                        "Maker Obligation Kinds in Order not all of the same kinds"
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
            if let Some(maker_bond_pct) = self.trade_details.content.maker_bond_pct {
                if maker_bond_pct == 0 {
                    return Err(N3xbError::Simple(format!(
                        "Trade Details Maker Bond Percentage should not be zero"
                    )));
                }
            } else {
                return Err(N3xbError::Simple(format!(
                    "Trade Details Maker Bond Percentage should be specified"
                )));
            }

            if let Some(taker_bond_pct) = self.trade_details.content.taker_bond_pct {
                if taker_bond_pct == 0 {
                    return Err(N3xbError::Simple(format!(
                        "Trade Details Taker Bond Percentage should not be zero"
                    )));
                }
            } else {
                return Err(N3xbError::Simple(format!(
                    "Trade Details Taker Bond Percentage should be specified"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_validate_order() {}

    #[tokio::test]
    async fn test_validate_order_maker_obligation_kinds_mismatches() {}

    #[tokio::test]
    async fn test_validate_order_maker_obligation_amount_zero() {}

    #[tokio::test]
    async fn test_validate_order_maker_obligation_amount_less_than_min() {}

    #[tokio::test]
    async fn test_validate_order_taker_obligation_kinds_mismatches() {}

    #[tokio::test]
    async fn test_validate_order_taker_obligation_under_specified() {}

    #[tokio::test]
    async fn test_validate_order_taker_obligation_limit_rate_zero() {}

    #[tokio::test]
    async fn test_validate_order_taker_obligation_limit_rate_negative() {}

    #[tokio::test]
    async fn test_validate_order_taker_obligation_market_offset() {}

    #[tokio::test]
    async fn test_validate_order_maker_bond_pct_missing() {}

    #[tokio::test]
    async fn test_validate_order_taker_bond_pct_missing() {}
}
