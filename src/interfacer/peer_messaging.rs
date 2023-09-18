use crate::common::types::{SerdeGenericTrait, SerdeGenericType};
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Debug, sync::Arc};
use uuid::Uuid;

// Peer Messaging Data Structure

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PeerMessage {
    pub(crate) peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    pub(crate) maker_order_note_id: String, // TODO: Is there a more specific type we can use here?
    pub(crate) trade_uuid: Uuid,            // TODO: Change to UUID type?
    pub(crate) message_type: SerdeGenericType,
    pub(crate) message: Arc<dyn SerdeGenericTrait>,
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
