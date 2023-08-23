use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, trace, warn};
use tokio::sync::{
    broadcast::{error::TryRecvError, Receiver},
    Mutex,
};
use tokio::task::JoinHandle;
use tokio::{task, time};

use crate::common::types::SerdeGenericTrait;
use crate::interface::{nostr::*, peer_messaging::PeerMessage};

pub struct Poller<OfferEngineSpecificType: SerdeGenericTrait> {
    arc_client: ArcClient,
    peer_message_callback: Option<fn(String, PeerMessage<OfferEngineSpecificType>)>,
}

impl<OfferEngineSpecificType: SerdeGenericTrait> Poller<OfferEngineSpecificType> {
    pub fn start(arc_client: ArcClient) -> JoinHandle<()> {
        task::spawn(async move {
            let mut poller: Poller<OfferEngineSpecificType> = Poller {
                arc_client,
                peer_message_callback: None,
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
        info!("Event: {}", event.as_json().to_string());
    }

    async fn process_notification_message(&self, relay_message: RelayMessage) {
        match relay_message {
            RelayMessage::Ok {
                event_id,
                status,
                message,
            } => {
                self.process_relay_message(event_id.to_string(), message)
                    .await;
            }
            RelayMessage::Event {
                subscription_id,
                event,
            } => {
                self.process_relay_message_event(*event).await;
            }
            _ => {
                debug!("Relay Message type fallthrough");
            }
        }
    }

    async fn process_relay_message(&self, id: String, message: String) {
        debug!("Relay Message: {}", message);
    }

    async fn process_relay_message_event(&self, event: Event) {
        debug!("Relay Message Event: {}", event.as_json().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[tokio::test]
    async fn test_process_event_notification() {}

    #[tokio::test]
    async fn test_process_message_notification() {}
}
