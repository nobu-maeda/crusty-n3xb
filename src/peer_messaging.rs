use erased_serde::Serialize as ErasedSerialize;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typetag;

// Peer Messaging Data Structure

#[typetag::serde(tag = "type")]
pub trait PeerMessageTrait: ErasedSerialize + Debug {}

#[derive(Debug, Deserialize, Serialize)]
pub struct PeerMessage {
    peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    maker_order_note_id: String,     // TODO: Is there a more specific type we can use here?
    trade_uuid: String,              // TODO: Change to UUID type?
    message: Box<dyn PeerMessageTrait>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PeerMessageContent {
    n3xb_peer_message: PeerMessage,
}
