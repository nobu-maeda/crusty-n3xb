use std::sync::Arc;
use std::time::Duration;

use log::{debug, info, trace, warn};
use tokio::sync::{
    broadcast::{error::TryRecvError, Receiver},
    Mutex,
};
use tokio::task::JoinHandle;
use tokio::{task, time};

use crate::interface::{nostr::*, peer_messaging::PeerMessage};
use crate::{common::types::SerdeGenericTrait, offer::Offer};

use super::peer_messaging::PeerMessageContent;

pub(crate) struct EventConfig<OfferEngineSpecificType: SerdeGenericTrait> {
    pub(crate) offer_callback: Box<dyn FnMut(PeerMessage<Offer<OfferEngineSpecificType>>) + Send>,
}

type ArcEventConfig<T> = Arc<Mutex<EventConfig<T>>>;

pub struct Poller<OfferEngineSpecificType: SerdeGenericTrait> {
    arc_client: ArcClient,
    arc_event_config: ArcEventConfig<OfferEngineSpecificType>,
}

impl<OfferEngineSpecificType: SerdeGenericTrait> Poller<OfferEngineSpecificType> {
    pub(crate) fn start(
        arc_client: ArcClient,
        arc_event_config: ArcEventConfig<OfferEngineSpecificType>,
    ) -> JoinHandle<()> {
        task::spawn(async move {
            let mut poller: Poller<OfferEngineSpecificType> = Poller {
                arc_client,
                arc_event_config,
            };
            poller.thread_main().await;
        })
    }

    async fn thread_main(&mut self) {
        let mut interval = time::interval(Duration::from_millis(100));
        let mut receiver: Option<Receiver<RelayPoolNotification>> = None;
        let mut some_pubkey: Option<String> = None;
        {
            let client = self.arc_client.lock().await;
            receiver = Some(client.notifications());
            some_pubkey = Some(client.keys().public_key().to_string());
        }
        let arc_receiver = Arc::new(Mutex::new(receiver.unwrap()));
        let pubkey = some_pubkey.unwrap();

        loop {
            trace!("Poller thread {} - Loop Awakened!", pubkey);

            // TODO: Pickup configurations or subscription updates

            // Lock needed resources
            let mut receiver = arc_receiver.lock().await;

            // Check receiver
            match receiver.try_recv() {
                Ok(notification) => {
                    match notification {
                        RelayPoolNotification::Event(url, event) => {
                            debug!(
                                "Poller thread {} received notification from url {} - Event",
                                pubkey,
                                url.to_string()
                            );
                            self.process_notification_event(event).await; // , Arc::clone(&arc_client)).await;
                        }
                        RelayPoolNotification::Message(url, message) => {
                            debug!(
                                "Poller thread {} received notification from url {} - Message",
                                pubkey,
                                url.to_string()
                            );
                            self.process_notification_message(message).await;
                        }
                        RelayPoolNotification::Shutdown => {
                            info!("Poller thread {} received notification - Shutdown", pubkey);
                        }
                    };
                }

                Err(error) => match error {
                    TryRecvError::Closed => {
                        warn!("Poller thread {} received error - Closed", pubkey)
                    }
                    TryRecvError::Empty => {
                        trace!("Poller thread {} received error - Empty", pubkey)
                    }
                    TryRecvError::Lagged(amount) => {
                        warn!(
                            "Poller thread {} received error - Lagged by {}",
                            pubkey, amount
                        )
                    }
                },
            }

            trace!("Poller thread {} - Loop Sleep...", pubkey);

            // Go back to sleep on the timer
            interval.tick().await;
        }
    }

    async fn process_notification_event(&self, event: Event) {
        if let Kind::EncryptedDirectMessage = event.kind {
            self.process_direct_message(event).await;
        } else {
            debug!("Event kind fallthrough");
        }
    }

    async fn process_direct_message(&self, event: Event) {
        let mut maybe_secret_key: Option<SecretKey> = None;
        {
            let client = self.arc_client.lock().await;
            maybe_secret_key = client.keys().secret_key().ok();
        }

        if let Ok(content) = decrypt(&maybe_secret_key.unwrap(), &event.pubkey, &event.content) {
            self.process_decrypted_direct_message(content).await;
        } else {
            println!("Failed to decrypt direct message");
        }
    }

    async fn process_decrypted_direct_message(&self, content: String) {
        match serde_json::from_str::<PeerMessageContent<Offer<OfferEngineSpecificType>>>(
            content.as_str(),
        ) {
            Ok(peer_message_content) => {
                let mut event_config = self.arc_event_config.lock().await;
                let peer_message = peer_message_content.n3xb_peer_message;
                (event_config.offer_callback)(peer_message);
            }
            Err(_) => {
                debug!(
                    "Failed to deserialize content as PeerMessage<OfferEngineSpecificType> {}",
                    content
                );
            }
        }
    }

    async fn process_notification_message(&self, relay_message: RelayMessage) {
        match relay_message {
            RelayMessage::Ok {
                event_id,
                status: _,
                message,
            } => {
                self.process_relay_message(event_id.to_string(), message)
                    .await;
            }
            RelayMessage::Event {
                subscription_id: _,
                event,
            } => {
                self.process_relay_message_event(*event).await;
            }
            _ => {
                debug!("Relay Message type fallthrough");
            }
        }
    }

    async fn process_relay_message(&self, _id: String, message: String) {
        debug!("Relay Message: {}", message);
    }

    async fn process_relay_message_event(&self, event: Event) {
        debug!("Relay Message Event: {}", event.as_json().to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, str::FromStr};

    use super::*;
    use crate::{interface::peer_messaging::*, testing::*};
    use nostr_sdk::secp256k1::schnorr::Signature;
    use std::sync::Mutex;
    use tokio::sync::broadcast;

    struct TestValues {
        trade_uuid: String,
    }

    struct TestRouter<OfferEngineSpecificType> {
        arc_test_values: Arc<Mutex<TestValues>>,
        _phantom_order_specifics: PhantomData<OfferEngineSpecificType>,
    }

    impl<OfferEngineSpecificType: SerdeGenericTrait> TestRouter<OfferEngineSpecificType> {
        fn new() -> Self {
            let test_values = TestValues {
                trade_uuid: "".to_string(),
            };
            TestRouter {
                arc_test_values: Arc::new(Mutex::new(test_values)),
                _phantom_order_specifics: PhantomData,
            }
        }

        fn offer_callback(
            &mut self,
            peer_message: PeerMessage<Offer<OfferEngineSpecificType>>,
        ) -> () {
            let mut test_values = self.arc_test_values.lock().unwrap();
            test_values.trade_uuid = peer_message.trade_uuid;
        }
    }

    #[tokio::test]
    async fn test_process_event_notification() {
        let keys = Keys::new(
            SecretKey::from_str("01010101010101010001020304050607ffff0000ffff00006363636363636363")
                .unwrap(),
        );

        let mut client: MockClient = Client::new();
        client.expect_keys().returning(|| {
            Keys::new(
                SecretKey::from_str(
                    "01010101010101010001020304050607ffff0000ffff00006363636363636363",
                )
                .unwrap(),
            )
        });

        let (sender, receiver) = broadcast::channel(1024);
        let mut maybe_receiver = Some(receiver);
        client
            .expect_notifications()
            .returning(move || maybe_receiver.take().unwrap());

        let arc_client = Arc::new(tokio::sync::Mutex::new(client));

        // Callback registration
        let router = TestRouter::<SomeTradeEngineTakerOfferSpecifics>::new();
        let arc_router = Arc::new(Mutex::new(router));
        let arc_router_clone = arc_router.clone();
        let event_config = EventConfig {
            offer_callback: Box::new(move |peer_message| {
                arc_router_clone
                    .lock()
                    .unwrap()
                    .offer_callback(peer_message)
            }),
        };
        let arc_event_config = Arc::new(tokio::sync::Mutex::new(event_config));

        let _handle = Poller::<SomeTradeEngineTakerOfferSpecifics>::start(
            arc_client,
            Arc::clone(&arc_event_config),
        );

        // Create Taker Offer to take the Order
        let offer = Offer {
            maker_obligation: SomeTestParams::offer_maker_obligation(),
            taker_obligation: SomeTestParams::offer_taker_obligation(),
            market_oracle_used: SomeTestParams::offer_marker_oracle_used(),
            trade_engine_specifics: SomeTradeEngineTakerOfferSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            },
            pow_difficulty: SomeTestParams::offer_pow_difficulty(),
        };

        let peer_message = PeerMessage {
            peer_message_id: None,
            maker_order_note_id: "".to_string(),
            trade_uuid: SomeTestParams::some_uuid_string(),
            message_type: PeerMessageType::TakerOffer,
            message: offer,
        };

        let peer_message_content = PeerMessageContent {
            n3xb_peer_message: peer_message,
        };

        let content_string = serde_json::to_string(&peer_message_content).unwrap();
        let encrypted_content = encrypt(
            &keys.secret_key().unwrap(),
            &keys.public_key(),
            content_string,
        )
        .unwrap();

        let event = Event {
            id: EventId::from_str(
                "ef537f25c895bfa782526529a9b63d97aa631564d5d789c2b765448c8635fb6c",
            )
            .unwrap(),
            pubkey: keys.public_key(),
            created_at: Timestamp::now(),
            kind: Kind::EncryptedDirectMessage,
            tags: [].to_vec(),
            content: encrypted_content,
            sig: Signature::from_str("14d0bf1a8953506fb460f58be141af767fd112535fb3922ef217308e2c26706f1eeb432b3dba9a01082f9e4d4ef5678ad0d9d532c0dfa907b568722d0b0119ba").unwrap()
        };

        let relay_pool_event =
            RelayPoolNotification::Event(Url::from_str("ws://localhost:8008/").unwrap(), event);

        // Use sender to trigger receive
        let _ = sender.send(relay_pool_event).unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check callback has been called
        let trade_uuid = arc_router
            .lock()
            .unwrap()
            .arc_test_values
            .lock()
            .unwrap()
            .trade_uuid
            .to_owned();

        assert_eq!(trade_uuid, SomeTestParams::some_uuid_string());

        // Wait for thread to shutdown
        // let thread_join = handle.join();
        // assert!(thread_join.is_ok());
    }

    #[tokio::test]
    async fn test_process_message_notification() {}
}
