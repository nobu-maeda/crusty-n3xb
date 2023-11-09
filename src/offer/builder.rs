use std::result::Result;

use crate::common::{error::N3xbError, types::SerdeGenericTrait};

use super::{Obligation, Offer};

pub struct OfferBuilder {
    maker_obligation: Option<Obligation>,
    taker_obligation: Option<Obligation>,
    market_oracle_used: Option<String>,
    trade_engine_specifics: Option<Box<dyn SerdeGenericTrait>>,
    pow_difficulty: Option<u64>,
}

impl OfferBuilder {
    pub fn new() -> Self {
        Self {
            maker_obligation: None,
            taker_obligation: None,
            market_oracle_used: None,
            trade_engine_specifics: None,
            pow_difficulty: None,
        }
    }

    pub fn maker_obligation(&mut self, maker_obligation: impl Into<Obligation>) -> &mut Self {
        self.maker_obligation = Some(maker_obligation.into());
        self
    }

    pub fn taker_obligation(&mut self, taker_obligation: impl Into<Obligation>) -> &mut Self {
        self.taker_obligation = Some(taker_obligation.into());
        self
    }

    pub fn market_oracle_used(&mut self, market_oracle_used: impl Into<String>) -> &mut Self {
        self.market_oracle_used = Some(market_oracle_used.into());
        self
    }

    pub fn trade_engine_specifics(
        &mut self,
        trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    ) -> &mut Self {
        self.trade_engine_specifics = Some(trade_engine_specifics);
        self
    }

    pub fn pow_difficulty(&mut self, pow_difficulty: impl Into<u64>) -> &mut Self {
        self.pow_difficulty = Some(pow_difficulty.into());
        self
    }

    pub fn build(&mut self) -> Result<Offer, N3xbError> {
        let Some(maker_obligation) = self.maker_obligation.as_ref() else {
            return Err(N3xbError::Simple("No Maker Obligations defined".to_string()));  // TODO: Error handling?
        };

        let Some(taker_obligation) = self.taker_obligation.as_ref() else {
            return Err(N3xbError::Simple("No Taker Obligations defined".to_string()));  // TODO: Error handling?
        };

        let Some(trade_engine_specifics) = self.trade_engine_specifics.take() else {
            return Err(N3xbError::Simple("No Trade Engine Specifics defined".to_string()));  // TODO: Error handling?
        };

        let offer = Offer {
            maker_obligation: maker_obligation.to_owned(),
            taker_obligation: taker_obligation.to_owned(),
            market_oracle_used: self.market_oracle_used.take(),
            trade_engine_specifics,
            pow_difficulty: self.pow_difficulty.take(),
            _private: (),
        };

        Ok(offer)
    }
}
