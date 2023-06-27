use crate::common::*;
use crate::error::*;
use super::{ maker_order::*, obligation::*, trade_details::*, trade_engine_details::* };

pub struct MakerOrderBuilder<'a> {  
  event_msg_client: &'a ArcClient,
  // DB

  // Trade Specific Parameters
  trade_uuid: Option<String>, // TODO: Change to UUID type
  maker_obligation: Option<MakerObligation>,
  taker_obligation: Option<TakerObligation>,
  trade_details: Option<TradeDetails>,
  engine_details: Option<TradeEngineDetails>,
  pow_difficulty: Option<u64>,
}

impl<'a> MakerOrderBuilder<'a> {
  pub fn new(
            event_msg_client: &'a ArcClient
             // DB
            ) -> Self {
    MakerOrderBuilder {
      event_msg_client,
      trade_uuid: Option::<String>::None, 
      maker_obligation: Option::<MakerObligation>::None, 
      taker_obligation: Option::<TakerObligation>::None, 
      trade_details: Option::<TradeDetails>::None, 
      engine_details: Option::<TradeEngineDetails>::None, 
      pow_difficulty: Option::<u64>::None
    }
  }

  pub fn trade_uuid(&mut self, trade_uuid: impl Into<String>) -> &mut Self {
    self.trade_uuid = Some(trade_uuid.into());
    self
  }

  pub fn maker_obligation(&mut self, maker_obligation: impl Into<MakerObligation>) -> &mut Self {
    self.maker_obligation = Some(maker_obligation.into());
    self
  }

  pub fn taker_obligation(&mut self, taker_obligation: impl Into<TakerObligation>) -> &mut Self {
    self.taker_obligation = Some(taker_obligation.into());
    self
  }

  pub fn trade_details(&mut self, trade_details: impl Into<TradeDetails>) -> &mut Self {
    self.trade_details = Some(trade_details.into());
    self
  }

  pub fn engine_details(&mut self, engine_details: impl Into<TradeEngineDetails>) -> &mut Self {
    self.engine_details = Some(engine_details.into());
    self
  }

  pub fn pow_difficulty(&mut self, pow_difficulty: impl Into<u64>) -> &mut Self {
    self.pow_difficulty = Some(pow_difficulty.into());
    self
  }

  pub fn build(&self) -> std::result::Result<MakerOrder, N3xbError> {
    let Some(trade_uuid) = self.trade_uuid.as_ref() else {
      return Err(N3xbError::Other("No Trade UUID".to_string()));  // TODO: Error handling?
    };

    let Some(maker_obligation) = self.maker_obligation.as_ref() else {
      return Err(N3xbError::Other("No Maker Obligations defined".to_string()));  // TODO: Error handling?
    };

    let Some(taker_obligation) = self.taker_obligation.as_ref() else {
      return Err(N3xbError::Other("No Taker Obligations defined".to_string()));  // TODO: Error handling?
    };

    let Some(trade_details) = self.trade_details.as_ref() else {
      return Err(N3xbError::Other("No Trade Details defined".to_string()));  // TODO: Error handling?
    };

    let Some(engine_details) = self.engine_details.as_ref() else {
      return Err(N3xbError::Other("No Engine Details defined".to_string()));  // TODO: Error handling?
    };

    let pow_difficulty = self.pow_difficulty.unwrap_or_else(|| 0);

    Ok(MakerOrder::new(self.event_msg_client,
      trade_uuid.to_owned(),
      maker_obligation.to_owned(),
      taker_obligation.to_owned(),
      trade_details.to_owned(),
      engine_details.to_owned(),
      pow_difficulty)
    )
  }
}