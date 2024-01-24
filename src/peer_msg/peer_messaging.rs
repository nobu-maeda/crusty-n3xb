use std::{any::Any, collections::HashSet, fmt::Debug};

use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::common::types::{EventIdString, SerdeGenericTrait, SerdeGenericType};

// Peer Messaging Data Structures

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerEnvelope {
    pub pubkey: XOnlyPublicKey,
    pub urls: HashSet<Url>,
    pub event_id: EventIdString,
    pub(crate) message_type: SerdeGenericType,
    pub message: Box<dyn SerdeGenericTrait>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PeerMessage {
    pub(crate) r#type: String,
    pub(crate) responding_to_id: Option<String>, // TODO: Is there a more specific type we can use here?
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
