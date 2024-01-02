use crate::common::{
    error::{N3xbError, OfferInvalidReason},
    types::{EventIdString, SerdeGenericTrait, SerdeGenericsPlaceholder},
};

use super::{TradeResponse, TradeResponseStatus};

pub struct TradeResponseBuilder {
    offer_event_id: Option<EventIdString>,
    trade_response: Option<TradeResponseStatus>,
    reject_reason: Vec<OfferInvalidReason>,
    trade_engine_specifics: Option<Box<dyn SerdeGenericTrait>>,
}

impl TradeResponseBuilder {
    pub fn new() -> Self {
        Self {
            offer_event_id: None,
            trade_response: None,
            reject_reason: [].to_vec(),
            trade_engine_specifics: None,
        }
    }

    pub fn offer_event_id(&mut self, offer_event_id: impl Into<EventIdString>) -> &mut Self {
        self.offer_event_id = Some(offer_event_id.into());
        self
    }

    pub fn trade_response(&mut self, trade_response: impl Into<TradeResponseStatus>) -> &mut Self {
        self.trade_response = Some(trade_response.into());
        self
    }

    pub fn reject_reason(&mut self, reject_reason: impl Into<OfferInvalidReason>) -> &mut Self {
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
        let Some(offer_event_id) = self.offer_event_id.as_ref() else {
            return Err(N3xbError::Simple("No Offer Event ID defined".to_string()));
            // TODO: Error handling?
        };

        let Some(trade_response) = self.trade_response.as_ref() else {
            return Err(N3xbError::Simple("No Trade Response defined".to_string()));
            // TODO: Error handling?
        };

        let trade_response = trade_response.to_owned();
        if self.reject_reason.is_empty() && trade_response == TradeResponseStatus::Rejected {
            return Err(N3xbError::Simple("No Reject Reason defined".to_string()));
            // TODO: Error handling?
        }

        let trade_engine_specifics =
            if let Some(trade_engine_specifics) = self.trade_engine_specifics.as_ref() {
                trade_engine_specifics.to_owned()
            } else {
                Box::new(SerdeGenericsPlaceholder {})
            };

        let trade_rsp = TradeResponse {
            offer_event_id: offer_event_id.to_owned(),
            trade_response: trade_response,
            reject_reason: self.reject_reason.to_owned(),
            trade_engine_specifics: trade_engine_specifics,
        };

        Ok(trade_rsp)
    }
}
