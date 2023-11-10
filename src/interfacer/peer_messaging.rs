use crate::common::types::{SerdeGenericTrait, SerdeGenericType};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Debug};
use uuid::Uuid;

// Peer Messaging Data Structures

pub(crate) struct PeerEnvelope {
    pub(crate) pubkey: XOnlyPublicKey,
    pub(crate) event_id: String,
    pub(crate) message_type: SerdeGenericType,
    pub(crate) message: Box<dyn SerdeGenericTrait>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PeerMessage {
    pub(crate) r#type: String,
    pub(crate) peer_message_id: Option<String>, // TODO: Is there a more specific type we can use here?
    pub(crate) maker_order_note_id: String, // TODO: Is there a more specific type we can use here?
    pub(crate) trade_uuid: Uuid,            // TODO: Change to UUID type?
    pub(crate) message_type: SerdeGenericType,
    pub(crate) message: Box<dyn SerdeGenericTrait>,
}

#[typetag::serde(name = "n3xB-peer-message")]
impl SerdeGenericTrait for PeerMessage {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}
