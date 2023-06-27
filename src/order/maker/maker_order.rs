use nostr_sdk::prelude::*;

use crate::common::*;
use super::super::Order;
use super::{ maker_order_note::*, obligation::*, trade_details::*, trade_engine_details::* };

pub struct MakerOrder<'a> {
  event_msg_client: &'a ArcClient,

  // Maker specific Order properties
  pub trade_uuid: String, // TODO: Change to UUID type
  pub maker_obligation: MakerObligation,
  pub taker_obligation: TakerObligation,
  pub trade_details: TradeDetails,
  pub engine_details: TradeEngineDetails,
  pub pow_difficulty: u64,
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
    let m_tag: Tag = Tag::Generic(TagKind::Custom("m".to_string()), self.maker_obligation.kind.to_tags().into_iter().collect());

    // Taker Obligation Tag #t
    let t_tag: Tag = Tag::Generic(TagKind::Custom("t".to_string()), self.taker_obligation.kind.to_tags().into_iter().collect());

    // Trade Detail Parameters #p
    let p_tag: Tag = Tag::Generic(TagKind::Custom("p".to_string()), self.trade_details.parameters_to_tags().into_iter().collect());

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