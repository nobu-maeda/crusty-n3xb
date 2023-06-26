use dyn_clone::DynClone;

use erased_serde::Serialize as ErasedSerialize;
use serde::{Serialize, Deserialize};
use typetag;

use nostr_sdk::prelude::*;

use iso_currency::Currency;

use crate::common::*;
use crate::error::*;

use std::collections::HashSet;
use std::fmt::*;
use std::fmt::Result as Result;

pub trait Order {
  // Common Order properties
  fn identifier() -> String;

  // Commands common to all orders
  fn message();
  fn remove();
  fn complete();
}

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

pub struct MakerOrder<'a> {
  event_msg_client: &'a ArcClient,

  // Maker specific Order properties
  trade_uuid: String, // TODO: Change to UUID type
  maker_obligation: MakerObligation,
  taker_obligation: TakerObligation,
  trade_details: TradeDetails,
  engine_details: TradeEngineDetails,
  pow_difficulty: u64,
}

impl<'a> MakerOrder<'a> {
  // Commands Maker can issue
  pub fn new(event_msg_client: &'a ArcClient,
             trade_uuid: String, // TODO: Change to UUID type
             maker_obligation: MakerObligation,
             taker_obligation: TakerObligation,
             trade_details: TradeDetails,
             engine_details: TradeEngineDetails,
             pow_difficulty: u64) -> Self {

    let maker_order = MakerOrder {
      event_msg_client,
      trade_uuid,
      maker_obligation,
      taker_obligation,
      trade_details,
      engine_details,
      pow_difficulty
    };

    // TODO: Add order to DB store 

    maker_order
  }

  pub fn list_offers() {

  }

  pub fn respond_to_offer() {

  }

  pub fn lock() {

  }

  async fn send_event_note(&self) {

    // Create Note Content
    let trade_engine_specifics = 
    if let Some(trade_engine_specifics) = self.engine_details.trade_engine_specifics.as_ref() {
      Some(trade_engine_specifics.to_owned())
    } else {
      None
    };

    let maker_order_note = MakerOrderNote {
      maker_obligation: self.maker_obligation.content.to_owned(),
      taker_obligation: self.taker_obligation.content.to_owned(),
      trade_details: self.trade_details.content.to_owned(),
      trade_engine_specifics,
      pow_difficulty: self.pow_difficulty,
    };

    let content_string = serde_json::to_string(&maker_order_note).unwrap();  // TODO: Error Handling?

    // Create Note Tags
    // TODO: Factor Custom Tag routine out to process a hash-map instead?

    // Trade UUID #i
    let i_tag: Tag = Tag::Generic(TagKind::Custom("i".to_string()), vec![self.trade_uuid.to_owned()]);

    // Maker Obligation Tag #m
    let m_tag: Tag = Tag::Generic(TagKind::Custom("m".to_string()), self.maker_obligation.kind.to_tags());

    // Taker Obligation Tag #t
    let t_tag: Tag = Tag::Generic(TagKind::Custom("t".to_string()), self.taker_obligation.kind.to_tags());

    // Trade Detail Parameters #p
    let p_tag: Tag = Tag::Generic(TagKind::Custom("p".to_string()), self.trade_details.parameters_to_tags());

    // Trade Engine Name #n
    let n_tag: Tag = Tag::Generic(TagKind::Custom("n".to_string()), vec![self.engine_details.trade_engine_name.to_string()]);

    // n3xB Event Kind #k
    let k_tag: Tag = Tag::Generic(TagKind::Custom("k".to_string()), vec!["maker-order".to_string()]);

    // NIP-78 Application Tag #d
    let d_tag: Tag = Tag::Generic(TagKind::Custom("d".to_string()), vec!["n3xb".to_string()]);

    let note_tags = [i_tag, m_tag, t_tag, p_tag, n_tag, k_tag, d_tag];

    // NIP-78 Event Kind - 30078
    let builder = EventBuilder::new(Kind::ParameterizedReplaceable(30078), content_string, &note_tags);

    let keys = self.event_msg_client.lock().unwrap().keys();
    self.event_msg_client.lock().unwrap().send_event(builder.to_event(&keys).unwrap()).await.unwrap();
  }
}

impl<'a> Order for MakerOrder<'a> {
  fn identifier() -> String {
    String::new()
  }

  fn message() {

  }

  fn remove() {

  }

  fn complete() {

  }
}

pub struct TakerOrder {
  // Taker specific Order properties
}

impl TakerOrder {
  // Commands Takers can issue
  pub fn take() {
    
  }
}

impl Order for TakerOrder {
  fn identifier() -> String {
    String::new()
  }

  fn message() {

  }

  fn remove() {

  }

  fn complete() {

  }
}

#[derive(Clone, Debug)]
pub enum ObligationKind {
  Bitcoin(Vec<BitcoinSettlementMethod>),
  Fiat(Currency, Vec<FiatPaymentMethod>),
  Custom(String),
}

impl ObligationKind {
  pub fn to_tags(&self) -> Vec<String> {
    let mut tag_string_vec: Vec<String>;
    match self {
      ObligationKind::Bitcoin(settlement_methods) => {
        let prefix_string = "ob-bitcoin";
        tag_string_vec = vec![prefix_string.to_string()];
        for settlment_method in settlement_methods {
          let tag_string = format!("{}-{}", prefix_string, settlment_method.to_string());
          tag_string_vec.push(tag_string.to_lowercase());
        }
      }

      ObligationKind::Fiat(currency, payment_methods) => {
        let prefix_string = "ob-fiat";
        let currency_prefix_string = format!("{}-{}", prefix_string, currency.code().to_string());
        tag_string_vec = vec![prefix_string.to_string(), currency_prefix_string.to_lowercase()];
        for payment_method in payment_methods {
          let tag_string = format!("{}-{}", currency_prefix_string, payment_method.to_string());
          tag_string_vec.push(tag_string.to_lowercase());
        }
      }

      ObligationKind::Custom(obligation_string) => {
        let prefix_string = "ob-custom";
        tag_string_vec = vec![prefix_string.to_string(), format!("{}-{}", prefix_string, obligation_string).to_lowercase()];
      }
    }
    tag_string_vec
  }
}

impl Display for ObligationKind {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      ObligationKind::Bitcoin(settlement_methods) => 
        write!(f, "Bitcoin {:?} Settlements", settlement_methods),
      ObligationKind::Fiat(currency, payment_methods) => 
        write!(f, "Fiat {:?} {:?} Settlements", currency, payment_methods),
      ObligationKind::Custom(obligation_string) => 
        write!(f, "Custom Settlement {:?}", obligation_string),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MakerObligationContent {
  amount: u64,
  amount_min: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TakerObligationContent{
  limit_rate: Option<f64>,
  market_offset_pct: Option<f64>,
  market_oracles: Option<Vec<String>>,  // TODO: Change to vector of URL type
}

#[derive(Clone, Debug)]
pub struct MakerObligation {
  kind: ObligationKind,
  content: MakerObligationContent,
}

#[derive(Clone, Debug)]
pub struct TakerObligation {
  kind: ObligationKind,
  content: TakerObligationContent,
}

#[derive(Clone, Debug)]
pub enum TradeTimeOutLimit {
  FourDays,
  OneDay,
  NoTimeout,
  TradeEngineSpecific,
}

impl TradeTimeOutLimit {
  pub fn to_tag(&self) -> String {
    match self {
      TradeTimeOutLimit::FourDays => "-4-days".to_string(),
      TradeTimeOutLimit::OneDay => "-24-hours".to_string(),
      TradeTimeOutLimit::NoTimeout => "-none".to_string(),
      TradeTimeOutLimit::TradeEngineSpecific => "".to_string(),
    }
  }
}

impl Display for TradeTimeOutLimit {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      TradeTimeOutLimit::FourDays => write!(f, "4 days"),
      TradeTimeOutLimit::OneDay => write!(f, "1 day"),
      TradeTimeOutLimit::NoTimeout => write!(f, "no timeout"),
      TradeTimeOutLimit::TradeEngineSpecific => write!(f, "trade engine specific"),
    }
  }
}

#[derive(Clone, Debug)]
pub enum TradeParameter {
  MakerHasReputation,
  TakerReputationRequired,
  BondsRequired,
  TrustedEscrow,
  TrustlessEscrow,
  TrustedArbitration,
  AcceptsPartialTake,
  TradeTimesOut(TradeTimeOutLimit)
}

impl TradeParameter {
  pub fn to_tag(&self) -> String {
    match self {
      TradeParameter::MakerHasReputation => "maker-has-reputation".to_string(),
      TradeParameter::TakerReputationRequired => "taker-reputation-required".to_string(),
      TradeParameter::BondsRequired => "bonds-required".to_string(),
      TradeParameter::TrustedEscrow => "trusted-escrow".to_string(),
      TradeParameter::TrustlessEscrow => "trustless-escrow".to_string(),
      TradeParameter::TrustedArbitration => "trusted-arbitration".to_string(),
      TradeParameter::AcceptsPartialTake => "accepts-partial-take".to_string(),
      TradeParameter::TradeTimesOut(timelimit) => format!("trade-times-out{}", timelimit.to_tag()),
    }
  }
}

impl Display for TradeParameter {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      TradeParameter::MakerHasReputation => write!(f, "Maker has reputation"),
      TradeParameter::TakerReputationRequired => write!(f, "Taker reputation required"),
      TradeParameter::BondsRequired => write!(f, "Bonds required"),
      TradeParameter::TrustedEscrow => write!(f, "Trusted escrow"),
      TradeParameter::TrustlessEscrow => write!(f, "Trustless escrow"),
      TradeParameter::TrustedArbitration => write!(f, "Trusted arbitration"),
      TradeParameter::AcceptsPartialTake => write!(f, "Accepts partial take"),
      TradeParameter::TradeTimesOut(timelimit) => write!(f, "Trade Times out with {:?} limit", timelimit),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeDetailsContent {
  maker_bond_pct: Option<u32>,
  taker_bond_pct: Option<u32>,
  trade_timeout: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct TradeDetails {
  parameters: HashSet<TradeParameter>,
  content: TradeDetailsContent,
}

impl TradeDetails {
  pub fn parameters_to_tags(&self) -> Vec<String> {
    let mut tag_string_vec: Vec<String> = vec![];
    for parameter in self.parameters.iter() {
      tag_string_vec.push(parameter.to_tag());
    }
    tag_string_vec
  }
}

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + DynClone + Debug {}

dyn_clone::clone_trait_object!(TradeEngineSpecfiicsTrait);

#[derive(Clone, Debug)]
pub struct TradeEngineDetails {
  trade_engine_name: String,
  trade_engine_specifics: Option<Box<dyn TradeEngineSpecfiicsTrait>>,
}

enum TradeStatus {

}

// n3xB Protocol JSON Deserialized Data Structures

#[derive(Debug, Deserialize, Serialize)]
pub struct MakerOrderNote {
  maker_obligation: MakerObligationContent,
  taker_obligation: TakerObligationContent,
  trade_details: TradeDetailsContent,
  trade_engine_specifics: Option<Box<dyn TradeEngineSpecfiicsTrait>>,
  pow_difficulty: u64,
}