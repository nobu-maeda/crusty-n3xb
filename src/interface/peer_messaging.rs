use crate::common::types::SerdeGenericTrait;
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Debug, rc::Rc};

// Peer Messaging Data Structure

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum PeerMessageType {
    TakerOffer,
    TradeResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PeerMessage {
    pub(crate) peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    pub(crate) maker_order_note_id: String, // TODO: Is there a more specific type we can use here?
    pub(crate) trade_uuid: String,          // TODO: Change to UUID type?
    pub(crate) message_type: PeerMessageType,
    pub(crate) message: Rc<dyn SerdeGenericTrait>,
}

#[typetag::serde(name = "n3xb_peer_message")]
impl SerdeGenericTrait for PeerMessage {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PeerMessageContent {
    pub(crate) n3xb_peer_message: PeerMessage,
}
