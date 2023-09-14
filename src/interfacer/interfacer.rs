use log::debug;
use std::net::SocketAddr;

use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey};
use tokio::sync::{mpsc, oneshot};

use super::nostr::*;

pub(crate) struct InterfacerHandle {
    tx: mpsc::Sender<InterfacerRequest>,
}

impl InterfacerHandle {
    pub(super) fn new(tx: mpsc::Sender<InterfacerRequest>) -> Self {
        InterfacerHandle { tx }
    }

    pub(crate) async fn get_public_key(&self) -> XOnlyPublicKey {
        let (rsp_tx, rsp_rx) = oneshot::channel::<XOnlyPublicKey>();
        let request = InterfacerRequest::GetPublicKey { rsp_tx };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()
    }

    pub(crate) async fn add_relays(
        &self,
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
    ) {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), Error>>();
        let request = InterfacerRequest::AddRelays {
            relays,
            connect,
            rsp_tx,
        };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        if let Some(error) = rsp_rx.await.unwrap().err() {
            panic!("Interfacer add_relays failed - {:?}", error); // TODO: Can handle this error better
        }
    }

    pub(crate) async fn remove_relay(&self, relay: impl Into<String>) {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Result<(), Error>>();
        let request = InterfacerRequest::RemoveRelay {
            relay: relay.into(),
            rsp_tx,
        };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        if let Some(error) = rsp_rx.await.unwrap().err() {
            panic!("Interfacer remove_relay failed - {:?}", error); // TODO: Can handle this error better
        }
    }

    pub(crate) async fn get_relays(&self) -> Vec<Url> {
        let (rsp_tx, rsp_rx) = oneshot::channel::<Vec<Url>>();
        let request = InterfacerRequest::GetRelays { rsp_tx };
        self.tx.send(request).await.unwrap(); // Oneshot should never fail
        rsp_rx.await.unwrap()
    }
}

pub(crate) struct Interfacer {
    tx: mpsc::Sender<InterfacerRequest>,
}

impl Interfacer {
    const INTEFACER_REQUEST_CHANNEL_SIZE: usize = 100;
    const NOSTR_EVENT_DEFAULT_POW_DIFFICULTY: u8 = 8;

    // Constructors

    pub(crate) async fn new(trade_engine_name: impl Into<String>) -> Self {
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut OsRng);
        Self::new_with_key(secret_key, trade_engine_name).await
    }

    pub(crate) async fn new_with_key(
        secret_key: SecretKey,
        trade_engine_name: impl Into<String>,
    ) -> Self {
        let client = Self::new_nostr_client(secret_key).await;
        Self::new_with_nostr_client(client, trade_engine_name).await
    }

    pub(super) async fn new_with_nostr_client(
        client: Client,
        trade_engine_name: impl Into<String>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<InterfacerRequest>(Self::INTEFACER_REQUEST_CHANNEL_SIZE);
        let mut actor = InterfacerActor::new(rx, trade_engine_name, client).await;
        tokio::spawn(async move { actor.run().await });
        Self { tx }
    }

    async fn new_nostr_client(secret_key: SecretKey) -> Client {
        let keys = Keys::new(secret_key);
        let opts = Options::new()
            .wait_for_connection(true)
            .wait_for_send(true)
            .difficulty(Self::NOSTR_EVENT_DEFAULT_POW_DIFFICULTY);
        let client = Client::with_opts(&keys, opts);
        // TODO: Add saved or default clients
        client.connect().await;
        client
    }

    // Interfacer Handle

    pub(crate) fn get_handle(&self) -> InterfacerHandle {
        InterfacerHandle::new(self.tx.clone())
    }
}

pub(super) enum InterfacerRequest {
    // Requests & Arguments
    GetPublicKey {
        rsp_tx: oneshot::Sender<XOnlyPublicKey>,
    },
    AddRelays {
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), Error>>,
    },
    RemoveRelay {
        relay: String,
        rsp_tx: oneshot::Sender<Result<(), Error>>,
    },
    GetRelays {
        rsp_tx: oneshot::Sender<Vec<Url>>,
    },
}
pub(super) struct InterfacerActor {
    rx: mpsc::Receiver<InterfacerRequest>,
    trade_engine_name: String,
    client: Client,
}

impl InterfacerActor {
    const MAKER_ORDER_NOTE_KIND: Kind = Kind::ParameterizedReplaceable(30078);

    pub(super) async fn new(
        rx: mpsc::Receiver<InterfacerRequest>,
        trade_engine_name: impl Into<String>,
        client: Client,
    ) -> Self {
        InterfacerActor {
            rx,
            trade_engine_name: trade_engine_name.into(),
            client,
        }
    }

    // Event Loop Main

    async fn run(&mut self) {
        // Nostr client initializaiton
        let pubkey = self.client.keys().public_key();
        self.client
            .subscribe(self.subscription_filters(pubkey))
            .await;

        // Request handling main event loop
        // !!! This function will end if no Sender remains for the Receiver
        while let Some(request) = self.rx.recv().await {
            self.handle_request(request).await;
        }
        debug!("Interfacer reqeust handling main loop ended");
    }

    async fn handle_request(&mut self, request: InterfacerRequest) {
        match request {
            InterfacerRequest::GetPublicKey { rsp_tx } => self.get_public_key(rsp_tx),

            // Change Relays
            InterfacerRequest::AddRelays {
                relays,
                connect,
                rsp_tx,
            } => self.add_relays(relays, connect, rsp_tx).await,

            InterfacerRequest::RemoveRelay { relay, rsp_tx } => {
                self.remove_relay(relay, rsp_tx).await
            }

            InterfacerRequest::GetRelays { rsp_tx } => self.get_relays(rsp_tx).await,
            // Change subscription filters

            // Send Maker Order Notes

            // Query Order Notes

            // Send Taker Offer Message
        }
    }

    // Nostr Client Management

    fn get_public_key(&self, rsp_tx: oneshot::Sender<XOnlyPublicKey>) {
        let public_key = self.client.keys().public_key();
        rsp_tx.send(public_key).unwrap(); // Oneshot should not fail
    }

    async fn add_relays(
        &mut self,
        relays: Vec<(impl Into<String> + 'static, Option<SocketAddr>)>,
        connect: bool,
        rsp_tx: oneshot::Sender<Result<(), Error>>,
    ) {
        if let Some(error) = self.client.add_relays(relays).await.err() {
            rsp_tx.send(Err(error)).unwrap(); // Oneshot should not fail
            return;
        }

        if connect {
            let pubkey = self.client.keys().public_key();
            self.client
                .subscribe(self.subscription_filters(pubkey))
                .await;
            self.client.connect().await;
        }
        rsp_tx.send(Ok(())).unwrap(); // Oneshot should not fail
    }

    async fn remove_relay(
        &mut self,
        relay: impl Into<String> + 'static,
        rsp_tx: oneshot::Sender<Result<(), Error>>,
    ) {
        rsp_tx.send(self.client.remove_relay(relay).await).unwrap(); // Oneshot should not fail
    }

    async fn get_relays(&self, rsp_tx: oneshot::Sender<Vec<Url>>) {
        let relays = self.client.relays().await;
        let urls: Vec<Url> = relays.iter().map(|(url, _)| url.to_owned()).collect();
        rsp_tx.send(urls).unwrap(); // Oneshot should not fail
    }

    fn subscription_filters(&self, pubkey: XOnlyPublicKey) -> Vec<Filter> {
        // Need a way to track existing Filters
        // Need a way to correlate State Machines to Subscriptions as to remove filters as necessary

        // Subscribe to all DM to own pubkey. Filter unrecognized DM out some other way. Can be spam prone
        let dm_filter = Filter::new().since(Timestamp::now()).pubkey(pubkey);
        vec![dm_filter]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[tokio::test]
    async fn test_get_public_key() {
        let mut client = Client::new();
        client.expect_keys().returning(|| {
            let secret_key = SomeTestParams::some_secret_key();
            Keys::new(secret_key)
        });
        client.expect_subscribe().returning(|_| {});

        let interfacer =
            Interfacer::new_with_nostr_client(client, SomeTestParams::engine_name_str()).await;
        let interfacer_handle = interfacer.get_handle();

        let secret_key = SomeTestParams::some_secret_key();
        let keys = Keys::new(secret_key);
        let public_key = keys.public_key();
        assert_eq!(public_key, interfacer_handle.get_public_key().await);
    }
}
