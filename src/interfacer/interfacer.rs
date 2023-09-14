use log::debug;

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
        let request: InterfacerRequest = InterfacerRequest::GetPublicKey { rsp_tx };
        self.tx.send(request).await.unwrap();
        rsp_rx.await.unwrap() // Oneshot should never fail
    }
}

pub(crate) struct Interfacer {
    tx: mpsc::Sender<InterfacerRequest>,
}

impl Interfacer {
    const INTEFACER_REQUEST_CHANNEL_SIZE: usize = 100;
    const NOSTR_EVENT_DEFAULT_POW_DIFFICULTY: u8 = 8;

    // Constructors

    pub(crate) async fn new(trade_engine_name: impl AsRef<str>) -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        Self::new_with_key(secret_key, trade_engine_name).await
    }

    pub(crate) async fn new_with_key(
        secret_key: SecretKey,
        trade_engine_name: impl AsRef<str>,
    ) -> Self {
        let client = Self::new_nostr_client(secret_key).await;
        Self::new_with_nostr_client(client, trade_engine_name).await
    }

    pub(super) async fn new_with_nostr_client(
        client: Client,
        trade_engine_name: impl AsRef<str>,
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
        trade_engine_name: impl AsRef<str>,
        client: Client,
    ) -> Self {
        InterfacerActor {
            rx,
            trade_engine_name: trade_engine_name.as_ref().to_owned(),
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
            self.handle_request(request);
        }
        debug!("Interfacer reqeust handling main loop ended");
    }

    fn handle_request(&mut self, request: InterfacerRequest) {
        match request {
            InterfacerRequest::GetPublicKey { rsp_tx } => self.get_public_key(rsp_tx),
            // Change Relays

            // Change subscription filters

            // Send Maker Order Notes

            // Query Order Notes

            // Send Taker Offer Message
        }
    }

    // Nostr Client Management

    fn get_public_key(&self, rsp_tx: oneshot::Sender<XOnlyPublicKey>) {
        let public_key = self.client.keys().public_key();
        rsp_tx.send(public_key).unwrap() // Oneshot should not fail;
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
