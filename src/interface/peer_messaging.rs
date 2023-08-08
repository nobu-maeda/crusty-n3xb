use crate::common::types::SerdeGenericTrait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Peer Messaging Data Structure

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerMessageType {
    TakerOffer,
    TradeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct PeerMessage<T: SerdeGenericTrait> {
    peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    maker_order_note_id: String,     // TODO: Is there a more specific type we can use here?
    trade_uuid: String,              // TODO: Change to UUID type?
    message_type: PeerMessageType,
    message: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct PeerMessageContent<T: SerdeGenericTrait> {
    n3xb_peer_message: PeerMessage<T>,
}
