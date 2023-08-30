use crate::common::types::SerdeGenericTrait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Peer Messaging Data Structure

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum PeerMessageType {
    TakerOffer,
    TradeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub(crate) struct PeerMessage<T: SerdeGenericTrait> {
    pub(crate) peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    pub(crate) maker_order_note_id: String, // TODO: Is there a more specific type we can use here?
    pub(crate) trade_uuid: String,          // TODO: Change to UUID type?
    pub(crate) message_type: PeerMessageType,
    pub(crate) message: T,
}

impl<T: SerdeGenericTrait> SerdeGenericTrait for PeerMessage<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub(crate) struct PeerMessageContent<T: SerdeGenericTrait> {
    pub(crate) n3xb_peer_message: PeerMessage<T>,
}
