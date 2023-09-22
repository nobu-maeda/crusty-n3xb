use log::{debug, error};
use std::collections::HashMap;

use tokio::sync::mpsc;

use uuid::Uuid;

use crate::common::{
    error::N3xbError,
    types::{SerdeGenericTrait, SerdeGenericType},
};

use super::peer_messaging::PeerMessage;

pub(super) struct Router {
    peer_message_tx_map:
        HashMap<Uuid, mpsc::Sender<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>>,
    fallback_tx: Option<mpsc::Sender<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>>,
}

impl Router {
    pub(super) fn new() -> Self {
        Router {
            peer_message_tx_map: HashMap::new(),
            fallback_tx: None,
        }
    }

    pub(super) fn register_trade_tx(
        &mut self,
        trade_uuid: Uuid,
        tx: mpsc::Sender<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>,
    ) {
        debug!("register_tx_for_trade_uuid() for {}", trade_uuid);
        if self.peer_message_tx_map.insert(trade_uuid, tx).is_some() {
            error!(
                "register_tx_for_trade_uuid() {} already registered",
                trade_uuid
            );
        };
    }

    pub(super) fn unregister_trade_tx(&mut self, trade_uuid: Uuid) {
        debug!("unregister_tx_for_trade_uuid() for {}", trade_uuid);
        if self.peer_message_tx_map.remove(&trade_uuid).is_none() {
            error!(
                "unregister_tx_for_trade_uuid() {} expected to already be registered",
                trade_uuid
            );
        }
    }

    pub(super) fn register_fallback_tx(
        &mut self,
        tx: mpsc::Sender<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>,
    ) {
        debug!("register_fallback_tx()");
        if self.fallback_tx.is_some() {
            error!("register_fallback_tx() already registered");
        }
        self.fallback_tx = Some(tx);
    }

    pub(super) fn unregister_fallback_tx(&mut self) {
        if self.fallback_tx.is_none() {
            error!("unregister_fallback_tx() expected to already be registered");
        } else {
            self.fallback_tx = None;
        }
    }

    pub(super) async fn handle_peer_message(
        &mut self,
        peer_message: PeerMessage,
    ) -> Result<(), N3xbError> {
        if let Some(tx) = self.peer_message_tx_map.get(&peer_message.trade_uuid) {
            tx.send((peer_message.message_type, peer_message.message))
                .await?;
            return Ok(());
        }

        if let Some(tx) = &self.fallback_tx {
            tx.send((peer_message.message_type, peer_message.message))
                .await?;
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
        offer::Offer,
        testing::{SomeTestParams, SomeTradeEngineTakerOfferSpecifics},
    };

    #[tokio::test]
    async fn test_tx_for_trade_uuid() {
        let trade_uuid = SomeTestParams::some_uuid();
        let mut router = Router::new();
        let (event_tx, mut event_rx) =
            mpsc::channel::<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>(1);
        let (fallback_tx, mut fallback_rx) =
            mpsc::channel::<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>(1);
        router.register_trade_tx(trade_uuid, event_tx);
        router.register_fallback_tx(fallback_tx);

        let offer = Offer {
            maker_obligation: SomeTestParams::offer_maker_obligation(),
            taker_obligation: SomeTestParams::offer_taker_obligation(),
            market_oracle_used: SomeTestParams::offer_marker_oracle_used(),
            trade_engine_specifics: Box::new(SomeTradeEngineTakerOfferSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            }),
            pow_difficulty: SomeTestParams::offer_pow_difficulty(),
        };

        let peer_message = PeerMessage {
            peer_message_id: Option::None,
            maker_order_note_id: "".to_string(),
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        router.handle_peer_message(peer_message).await.unwrap();

        let mut event_count = 0;
        let mut fallback_count = 0;

        while let Some(event) = event_rx.try_recv().ok() {
            let (serde_type, serde_message) = event;
            match serde_type {
                SerdeGenericType::TakerOffer => {
                    let _ = serde_message.downcast_ref::<Offer>().unwrap();
                    event_count += 1;
                }
                _ => {
                    panic!("Unexpected serde type {:?} from rx", serde_type);
                }
            }
        }

        while let Some(event) = fallback_rx.try_recv().ok() {
            let (serde_type, serde_message) = event;
            match serde_type {
                SerdeGenericType::TakerOffer => {
                    let _ = serde_message.downcast_ref::<Offer>().unwrap();
                    fallback_count += 1;
                }
                _ => {
                    panic!("Unexpected serde type {:?} from rx", serde_type);
                }
            }
        }

        assert_eq!(1, event_count);
        assert_eq!(0, fallback_count);
    }

    #[tokio::test]
    async fn test_fallback_tx() {
        let trade_uuid = SomeTestParams::some_uuid();
        let mut router = Router::new();
        let (event_tx, mut event_rx) =
            mpsc::channel::<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>(1);
        let (fallback_tx, mut fallback_rx) =
            mpsc::channel::<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>(1);
        router.register_trade_tx(Uuid::new_v4(), event_tx);
        router.register_fallback_tx(fallback_tx);

        let offer = Offer {
            maker_obligation: SomeTestParams::offer_maker_obligation(),
            taker_obligation: SomeTestParams::offer_taker_obligation(),
            market_oracle_used: SomeTestParams::offer_marker_oracle_used(),
            trade_engine_specifics: Box::new(SomeTradeEngineTakerOfferSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            }),
            pow_difficulty: SomeTestParams::offer_pow_difficulty(),
        };

        let peer_message = PeerMessage {
            peer_message_id: Option::None,
            maker_order_note_id: "".to_string(),
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        router.handle_peer_message(peer_message).await.unwrap();

        let mut event_count = 0;
        let mut fallback_count = 0;

        while let Some(event) = event_rx.try_recv().ok() {
            let (serde_type, serde_message) = event;
            match serde_type {
                SerdeGenericType::TakerOffer => {
                    let _ = serde_message.downcast_ref::<Offer>().unwrap();
                    event_count += 1;
                }
                _ => {
                    panic!("Unexpected serde type {:?} from rx", serde_type);
                }
            }
        }

        while let Some(event) = fallback_rx.try_recv().ok() {
            let (serde_type, serde_message) = event;
            match serde_type {
                SerdeGenericType::TakerOffer => {
                    let _ = serde_message.downcast_ref::<Offer>().unwrap();
                    fallback_count += 1;
                }
                _ => {
                    panic!("Unexpected serde type {:?} from rx", serde_type);
                }
            }
        }

        assert_eq!(0, event_count);
        assert_eq!(1, fallback_count);
    }

    #[tokio::test]
    async fn test_no_matching_registered_tx() {
        let trade_uuid = SomeTestParams::some_uuid();
        let mut router = Router::new();
        let (event_tx, mut event_rx) =
            mpsc::channel::<(SerdeGenericType, Box<dyn SerdeGenericTrait>)>(1);
        router.register_trade_tx(Uuid::new_v4(), event_tx);

        let offer = Offer {
            maker_obligation: SomeTestParams::offer_maker_obligation(),
            taker_obligation: SomeTestParams::offer_taker_obligation(),
            market_oracle_used: SomeTestParams::offer_marker_oracle_used(),
            trade_engine_specifics: Box::new(SomeTradeEngineTakerOfferSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            }),
            pow_difficulty: SomeTestParams::offer_pow_difficulty(),
        };

        let peer_message = PeerMessage {
            peer_message_id: Option::None,
            maker_order_note_id: "".to_string(),
            trade_uuid,
            message_type: SerdeGenericType::TakerOffer,
            message: Box::new(offer),
        };

        let result = router.handle_peer_message(peer_message).await;

        let mut event_count = 0;
        let fallback_count = 0;

        while let Some(event) = event_rx.try_recv().ok() {
            let (serde_type, serde_message) = event;
            match serde_type {
                SerdeGenericType::TakerOffer => {
                    let _ = serde_message.downcast_ref::<Offer>().unwrap();
                    event_count += 1;
                }
                _ => {
                    panic!("Unexpected serde type {:?} from rx", serde_type);
                }
            }
        }

        assert!(result.is_err());
        assert_eq!(0, event_count);
        assert_eq!(0, fallback_count);
    }
}
