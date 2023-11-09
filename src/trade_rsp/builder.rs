use uuid::Uuid;

use crate::common::{error::N3xbError, types::SerdeGenericTrait};

use super::{TradeRejectReason, TradeResponse, TradeResponseStatus};

pub struct TradeResponseBuilder {
    offer_uuid: Option<Uuid>,
    trade_response: Option<TradeResponseStatus>,
    reject_reason: Vec<TradeRejectReason>,
    trade_engine_specifics: Option<Box<dyn SerdeGenericTrait>>,
}

impl TradeResponseBuilder {
    pub fn new() -> Self {
        Self {
            offer_uuid: None,
            trade_response: None,
            reject_reason: [].to_vec(),
            trade_engine_specifics: None,
        }
    }

    pub fn offer_uuid(&mut self, offer_uuid: impl Into<Uuid>) -> &mut Self {
        self.offer_uuid = Some(offer_uuid.into());
        self
    }

    pub fn trade_response(&mut self, trade_response: impl Into<TradeResponseStatus>) -> &mut Self {
        self.trade_response = Some(trade_response.into());
        self
    }

    pub fn reject_reason(&mut self, reject_reason: impl Into<TradeRejectReason>) -> &mut Self {
        self.reject_reason.push(reject_reason.into());
        self
    }

    pub fn trade_engine_specifics(
        &mut self,
        trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    ) -> &mut Self {
        self.trade_engine_specifics = Some(trade_engine_specifics);
        self
    }

    pub fn build(&self) -> Result<TradeResponse, N3xbError> {
        let Some(offer_uuid) = self.offer_uuid.as_ref() else {
            return Err(N3xbError::Simple("No Offer UUID defined".to_string()));  // TODO: Error handling?
        };

        let Some(trade_response) = self.trade_response.as_ref() else {
            return Err(N3xbError::Simple("No Trade Response defined".to_string()));  // TODO: Error handling?
        };

        let trade_response = trade_response.to_owned();
        if self.reject_reason.is_empty() && trade_response == TradeResponseStatus::Rejected {
            return Err(N3xbError::Simple("No Reject Reason defined".to_string()));
            // TODO: Error handling?
        }

        let Some(trade_engine_specifics) = self.trade_engine_specifics.as_ref() else {
            return Err(N3xbError::Simple("No Trade Engine Specifics defined".to_string()));
        };

        let trade_rsp = TradeResponse {
            offer_uuid: offer_uuid.to_owned(),
            trade_response: trade_response,
            reject_reason: self.reject_reason.to_owned(),
            trade_engine_specifics: trade_engine_specifics.to_owned(),
        };

        Ok(trade_rsp)
    }
}
