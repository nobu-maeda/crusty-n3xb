use dyn_clone::DynClone;

use erased_serde::Serialize as ErasedSerialize;
use serde::{Serialize, Deserialize};
use std::fmt::*;
use typetag;

use iso_currency::Currency;

use crate::common::*;

use std::collections::HashSet;

pub trait Order {
  // Common Order properties
  fn identifier() -> String;

  // Commands common to all orders
  fn message();
  fn remove();
  fn complete();
}

pub struct MakerOrder {
  // Maker specific Order properties
  trade_uuid: String, // TODO: Change to UUID type
  maker_obligation: MakerObligation,
  taker_obligation: TakerObligation,
  trade_details: TradeDetails,
  trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
  pow_difficulty: u64,
}

impl MakerOrder {
  // Commands Maker can issue
  pub fn new(trade_uuid: String, // TODO: Change to UUID type
             maker_obligation: MakerObligation,
             taker_obligation: TakerObligation,
             trade_details: TradeDetails,
             trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
             pow_difficlty: u64) -> MakerOrder {

    let maker_order = MakerOrder {
      trade_uuid: trade_uuid,
      maker_obligation: maker_obligation.clone(),
      taker_obligation: taker_obligation.clone(),
      trade_details: trade_details.clone(),
      trade_engine_specifics: dyn_clone::clone_box(&*trade_engine_specifics),
      pow_difficulty: pow_difficlty
    };

    let maker_order_note = MakerOrderNote {
      maker_obligation: maker_obligation.content.clone(),
      taker_obligation: taker_obligation.content.clone(),
      trade_details: trade_details.content.clone(),
      trade_engine_specifics: dyn_clone::clone_box(&*trade_engine_specifics),
      pow_difficlty: pow_difficlty,
    };

    // Add order to DB store 

    // Use Nostr module to send as note?

    // Use notification module to notify async completion? Or just call back?

    maker_order
  }

  pub fn list_offers() {

  }

  pub fn respond_to_offer() {

  }

  pub fn lock() {

  }
}

impl Order for MakerOrder {
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
pub enum ObligationKinds {
  Bitcoin(BitcoinSettlementTypes),
  Fiat(Currency, FiatPaymentMethods),
  Custom(String),
}

impl Display for ObligationKinds {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      ObligationKinds::Bitcoin(settlement_type) => 
        write!(f, "Bitcoin {:?} Settlement", settlement_type),
      ObligationKinds::Fiat(currency, payment_method) => 
        write!(f, "Fiat {:?} {:?} Settlement", currency, payment_method),
      ObligationKinds::Custom(obligation_string) => 
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
  kind: ObligationKinds,
  content: MakerObligationContent,
}

#[derive(Clone, Debug)]
pub struct TakerObligation {
  kind: ObligationKinds,
  content: TakerObligationContent,
}

#[derive(Clone, Debug)]
pub enum TradeTimeOutLimit {
  FourDays,
  OneDay,
  NoTimeout,
  TradeEngineSpecific,
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
pub enum TradeParameters {
  MakerHasReputation,
  TakerReputationRequired,
  BondsRequired,
  TrustedEscrow,
  TrustlessEscrow,
  TrustedArbitration,
  AcceptsPartialTake,
  TradeTimesOut(TradeTimeOutLimit)
}

impl Display for TradeParameters {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      TradeParameters::MakerHasReputation => write!(f, "Maker has reputation"),
      TradeParameters::TakerReputationRequired => write!(f, "Taker reputation required"),
      TradeParameters::BondsRequired => write!(f, "Bonds required"),
      TradeParameters::TrustedEscrow => write!(f, "Trusted escrow"),
      TradeParameters::TrustlessEscrow => write!(f, "Trustless escrow"),
      TradeParameters::TrustedArbitration => write!(f, "Trusted arbitration"),
      TradeParameters::AcceptsPartialTake => write!(f, "Accepts partial take"),
      TradeParameters::TradeTimesOut(timelimit) => write!(f, "Trade Times out with {:?} limit", timelimit),
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
  parameters: HashSet<TradeParameters>,
  content: TradeDetailsContent,
}

#[typetag::serde(tag = "type")]
pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + DynClone + Debug {}

dyn_clone::clone_trait_object!(TradeEngineSpecfiicsTrait);

pub struct TradeEngineSpecfiics {
  trade_engine_name: String,
  trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
}

enum TradeStatus {

}

// TODO: NIP-12 Tagging implementation

// n3xB Protocol JSON Deserialized Data Structures

#[derive(Debug, Deserialize, Serialize)]
pub struct MakerOrderNote {
  maker_obligation: MakerObligationContent,
  taker_obligation: TakerObligationContent,
  trade_details: TradeDetailsContent,
  trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
  pow_difficlty: u64,
}