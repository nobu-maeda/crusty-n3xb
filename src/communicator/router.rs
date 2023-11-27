use log::debug;
use std::collections::HashMap;

use secp256k1::XOnlyPublicKey;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    common::{error::N3xbError, types::EventIdString},
    peer_msg::{PeerEnvelope, PeerMessage},
};

pub(super) struct Router {
    peer_message_tx_map: HashMap<Uuid, mpsc::Sender<PeerEnvelope>>,
    peer_message_fallback_tx: Option<mpsc::Sender<PeerEnvelope>>,
}

impl Router {
    pub(super) fn new() -> Self {
        Router {
            peer_message_tx_map: HashMap::new(),
            peer_message_fallback_tx: None,
        }
    }

    pub(super) fn register_peer_message_tx(
        &mut self,
        trade_uuid: Uuid,
        tx: mpsc::Sender<PeerEnvelope>,
    ) -> Result<(), N3xbError> {
        debug!("register_tx_for_trade_uuid() for {}", trade_uuid);
        if self.peer_message_tx_map.insert(trade_uuid, tx).is_some() {
            let error = N3xbError::Simple(format!(
                "register_tx_for_trade_uuid() for {} already registered",
                trade_uuid
            ));
            Err(error)
        } else {
            Ok(())
        }
    }

    pub(super) fn unregister_peer_message_tx(&mut self, trade_uuid: Uuid) -> Result<(), N3xbError> {
        debug!("unregister_tx_for_trade_uuid() for {}", trade_uuid);
        if self.peer_message_tx_map.remove(&trade_uuid).is_none() {
            let error = N3xbError::Simple(format!(
                "unregister_tx_for_trade_uuid() {} expected to already be registered",
                trade_uuid
            ));
            Err(error)
        } else {
            Ok(())
        }
    }

    pub(super) fn register_peer_message_fallback_tx(
        &mut self,
        tx: mpsc::Sender<PeerEnvelope>,
    ) -> Result<(), N3xbError> {
        debug!("register_peer_message_fallback_tx()");

        let mut result = Ok(());
        if self.peer_message_fallback_tx.is_some() {
            let error = N3xbError::Simple(format!(
                "register_peer_message_fallback_tx() already registered"
            ));
            result = Err(error);
        }
        self.peer_message_fallback_tx = Some(tx);
        result
    }

    pub(super) fn unregister_peer_message_fallback_tx(&mut self) -> Result<(), N3xbError> {
        debug!("unregister_peer_message_fallback_tx()");

        let mut result = Ok(());
        if self.peer_message_fallback_tx.is_none() {
            let error = N3xbError::Simple(format!(
                "unregister_peer_message_fallback_tx() expected to already be registered"
            ));
            result = Err(error);
        }
        self.peer_message_fallback_tx = None;
        result
    }

    pub(super) async fn handle_peer_message(
        &mut self,
        pubkey: XOnlyPublicKey,
        event_id: EventIdString,
        peer_message: PeerMessage,
    ) -> Result<(), N3xbError> {
        let envelope = PeerEnvelope {
            pubkey,
            event_id,
            message_type: peer_message.message_type.clone(),
            message: peer_message.message.clone(),
        };

        if let Some(tx) = self.peer_message_tx_map.get(&peer_message.trade_uuid) {
            tx.send(envelope).await?;
            return Ok(());
        }

        if let Some(tx) = &self.peer_message_fallback_tx {
            tx.send(envelope).await?;
            return Ok(());
        }

        Err(N3xbError::Simple(
            "No channel Tx registered for peer message routing".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::types::SerdeGenericType,
        offer::Offer,
        testing::{SomeTestOfferParams, SomeTestOrderParams},
    };

    #[tokio::test]
    async fn test_tx_for_trade_uuid() {
        let trade_uuid = SomeTestOrderParams::some_uuid();
        let mut router = Router::new();
        let (event_tx, mut event_rx) = mpsc::channel::<PeerEnvelope>(1);
        let (peer_message_fallback_tx, mut fallback_rx) = mpsc::channel::<PeerEnvelope>(1);
        router
            .register_peer_message_tx(trade_uuid, event_tx)
            .unwrap();
        router
            .register_peer_message_fallback_tx(peer_message_fallback_tx)
            .unwrap();

        let offer = SomeTestOfferParams::default_builder().build().unwrap();

        let peer_message = PeerMessage {
            r#type: "n3xb-peer-message".to_string(),
            responding_to_id: Option::None,
            maker_order_note_id: "".to_string(),
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        router
            .handle_peer_message(
                SomeTestOfferParams::some_x_only_public_key(),
                "".to_string(),
                peer_message,
            )
            .await
            .unwrap();

        let mut event_count = 0;
        let mut fallback_count = 0;

        while let Some(peer_envelope) = event_rx.try_recv().ok() {
            match peer_envelope.message_type {
                SerdeGenericType::TakerOffer => {
                    let _ = peer_envelope.message.downcast_ref::<Offer>().unwrap();
                    event_count += 1;
                }
                _ => {
                    panic!(
                        "Unexpected serde type {:?} from rx",
                        peer_envelope.message_type
                    );
                }
            }
        }

        while let Some(peer_envelope) = fallback_rx.try_recv().ok() {
            match peer_envelope.message_type {
                SerdeGenericType::TakerOffer => {
                    let _ = peer_envelope.message.downcast_ref::<Offer>().unwrap();
                    fallback_count += 1;
                }
                _ => {
                    panic!(
                        "Unexpected serde type {:?} from rx",
                        peer_envelope.message_type
                    );
                }
            }
        }

        assert_eq!(1, event_count);
        assert_eq!(0, fallback_count);
    }

    #[tokio::test]
    async fn test_peer_message_fallback_tx() {
        let trade_uuid = SomeTestOrderParams::some_uuid();
        let mut router = Router::new();
        let (event_tx, mut event_rx) = mpsc::channel::<PeerEnvelope>(1);
        let (peer_message_fallback_tx, mut fallback_rx) = mpsc::channel::<PeerEnvelope>(1);
        router
            .register_peer_message_tx(Uuid::new_v4(), event_tx)
            .unwrap();
        router
            .register_peer_message_fallback_tx(peer_message_fallback_tx)
            .unwrap();

        let offer = SomeTestOfferParams::default_builder().build().unwrap();

        let peer_message = PeerMessage {
            r#type: "n3xb-peer-message".to_string(),
            responding_to_id: Option::None,
            maker_order_note_id: "".to_string(),
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        router
            .handle_peer_message(
                SomeTestOfferParams::some_x_only_public_key(),
                "".to_string(),
                peer_message,
            )
            .await
            .unwrap();

        let mut event_count = 0;
        let mut fallback_count = 0;

        while let Some(peer_envelope) = event_rx.try_recv().ok() {
            match peer_envelope.message_type {
                SerdeGenericType::TakerOffer => {
                    let _ = peer_envelope.message.downcast_ref::<Offer>().unwrap();
                    event_count += 1;
                }
                _ => {
                    panic!(
                        "Unexpected serde type {:?} from rx",
                        peer_envelope.message_type
                    );
                }
            }
        }

        while let Some(peer_envelope) = fallback_rx.try_recv().ok() {
            match peer_envelope.message_type {
                SerdeGenericType::TakerOffer => {
                    let _ = peer_envelope.message.downcast_ref::<Offer>().unwrap();
                    fallback_count += 1;
                }
                _ => {
                    panic!(
                        "Unexpected serde type {:?} from rx",
                        peer_envelope.message_type
                    );
                }
            }
        }

        assert_eq!(0, event_count);
        assert_eq!(1, fallback_count);
    }

    #[tokio::test]
    async fn test_no_matching_registered_tx() {
        let trade_uuid = SomeTestOrderParams::some_uuid();
        let mut router = Router::new();
        let (event_tx, mut event_rx) = mpsc::channel::<PeerEnvelope>(1);
        router
            .register_peer_message_tx(Uuid::new_v4(), event_tx)
            .unwrap();

        let offer = SomeTestOfferParams::default_builder().build().unwrap();

        let peer_message = PeerMessage {
            r#type: "n3xb-peer-message".to_string(),
            responding_to_id: Option::None,
            maker_order_note_id: "".to_string(),
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        let result = router
            .handle_peer_message(
                SomeTestOfferParams::some_x_only_public_key(),
                "".to_string(),
                peer_message,
            )
            .await;

        let mut event_count = 0;
        let fallback_count = 0;

        while let Some(peer_envelope) = event_rx.try_recv().ok() {
            match peer_envelope.message_type {
                SerdeGenericType::TakerOffer => {
                    let _ = peer_envelope.message.downcast_ref::<Offer>().unwrap();
                    event_count += 1;
                }
                _ => {
                    panic!(
                        "Unexpected serde type {:?} from rx",
                        peer_envelope.message_type
                    );
                }
            }
        }

        assert!(result.is_err());
        assert_eq!(0, event_count);
        assert_eq!(0, fallback_count);
    }
}
