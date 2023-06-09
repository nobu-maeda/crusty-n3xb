pub trait Order {
  // Common Order properties

  // Order Note - Encodable as JSON
}

pub struct MakerOrder {
  // Maker specific Order properties

  // Commands Maker can issue
}

pub struct TakerOrder {
  // Taker specific Order properties

  // Commands Takers can issue
}

// n3xB Protocol JSON Deserialized Data Structures

// Maker Order Note Content Data Structure

pub mod maker_order_note {
  use erased_serde::Serialize as ErasedSerialize;
  use serde::{Serialize, Deserialize};
  use std::fmt::Debug;
  use typetag;

  // TODO: NIP-12 Tagging implementation

  #[derive(Debug, Deserialize, Serialize)]
  pub struct MakerObligation {
    amount: u64,
    amount_min: Option<u64>,
  }

  #[derive(Debug, Deserialize, Serialize)]
  pub struct TakerObligation {
    limit_rate: Option<f64>,
    market_offset_pct: Option<f64>,
    market_oracles: Option<Vec<String>>,  // TODO: Change to vector of URL type
  }

  #[derive(Debug, Deserialize, Serialize)]
  pub struct TradeDetails {
    maker_bond_pct: Option<u32>,
    taker_bond_pct: Option<u32>,
    trade_timeout: Option<u32>,
  }

  #[typetag::serde(tag = "type")]
  pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + Debug {}

  #[derive(Debug, Deserialize, Serialize)]
  pub struct MakerOrderNoteContent {
    maker_obligation: MakerObligation,
    taker_obligation: TakerObligation,
    trade_details: TradeDetails,
    trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
    pow_difficlty: u64,
  }
}

// Peer Messaging Data Structure

pub mod peer_messaging {
  use erased_serde::Serialize as ErasedSerialize;
  use serde::{Serialize, Deserialize};
  use std::fmt::Debug;
  use typetag;

  #[typetag::serde(tag = "type")]
  pub trait PeerMessageTrait: ErasedSerialize + Debug {}

  #[derive(Debug, Deserialize, Serialize)]
  pub struct PeerMessage {
    peer_message_id: Option<String>,  // TODO: Is there a more specific type we can use here?
    maker_order_note_id: String,  // TODO: Is there a more specific type we can use here?
    trade_uuid: String,  // TODO: Change to UUID type?
    message: Box<dyn PeerMessageTrait>,
  }

  #[derive(Debug, Deserialize, Serialize)]
  pub struct PeerMessageContent {
    n3xb_peer_message: PeerMessage,
  }
}

// Take Order Message Data Structure

pub mod take_order_message {
  use erased_serde::Serialize as ErasedSerialize;
  use serde::{Serialize, Deserialize};
  use std::fmt::Debug;
  use typetag;

  #[derive(Debug, Deserialize, Serialize)]
  pub struct Obligation {
    amount: u64,
    currency: String,  // TODO: Change to ISO 4217 Enum
    payment: String,  // TODO: Change to Payment Method Enum
    bond_amount: Option<u64>,
  }

  #[typetag::serde(tag = "type")]
  pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + Debug {}

  #[derive(Debug, Deserialize, Serialize)]
  pub struct TakeOrderMessage {
    maker_obligation: Obligation,
    taker_obligation: Obligation,
    market_oracle_used: Option<String>,  // TODO: Change to URL type
    trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,
    pow_difficulty: u64,
  }

  #[typetag::serde(name = "n3xB-take-order")]
  impl super::peer_messaging::PeerMessageTrait for TakeOrderMessage {}
}

// Trade Response Message Data Structure

pub mod trade_response_message {
  use erased_serde::Serialize as ErasedSerialize;
  use serde::{Serialize, Deserialize};
  use std::fmt::Debug;
  use typetag;

  #[typetag::serde(tag = "type")]
  pub trait TradeEngineSpecfiicsTrait: ErasedSerialize + Debug {}

  #[derive(Debug, Deserialize, Serialize)]
  pub struct TradeResponseMessage {
    trade_response: String,  // TODO: Change to Enums
    reject_reason: Option<Vec<String>>,  // TODO: Change to Enums
    trade_engine_specifics: Box<dyn TradeEngineSpecfiicsTrait>,  
  }

  #[typetag::serde(name = "n3xB-trade-response")]
  impl super::peer_messaging::PeerMessageTrait for TradeResponseMessage {}
}