use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typetag;

// Peer Messaging Data Structure

#[typetag::serialize(tag = "type")]
pub trait PeerMessageTrait: Debug {}

#[derive(Debug, Deserialize, Serialize)]
pub struct PeerMessage<T: PeerMessageTrait + Clone + Serialize> {
    peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    maker_order_note_id: String,     // TODO: Is there a more specific type we can use here?
    trade_uuid: String,              // TODO: Change to UUID type?
    message: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PeerMessageContent<T: PeerMessageTrait + Clone + Serialize> {
    n3xb_peer_message: PeerMessage<T>,
}
