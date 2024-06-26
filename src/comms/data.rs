use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use tracing::debug;

use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

use crate::common::{
    error::N3xbError,
    persist::Persister,
    types::{BitcoinNetwork, SerdeGenericTrait},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommsDataStore {
    relays: HashMap<url::Url, Option<SocketAddr>>,
    // filters:
    event_ids: HashSet<String>,
}

#[typetag::serde(name = "n3xb_comms_data")]
impl SerdeGenericTrait for CommsDataStore {
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

pub(crate) struct CommsData {
    store: Arc<RwLock<CommsDataStore>>,
    persister: Persister,
}

impl CommsData {
    pub(crate) fn new(
        dir_path: impl AsRef<Path>,
        pubkey: XOnlyPublicKey,
        trade_engine_name: impl AsRef<str>,
        network: impl Borrow<BitcoinNetwork>,
    ) -> Result<Self, N3xbError> {
        let data_path = Self::setup_data_path(&dir_path, pubkey, trade_engine_name, network)?;

        let mut store = CommsDataStore {
            relays: HashMap::new(),
            event_ids: HashSet::new(),
        };

        if data_path.exists() {
            match Self::restore(&data_path) {
                Ok(restored_data) => {
                    store = restored_data;
                }
                Err(err) => {
                    panic!(
                        "Comms w/ Pubkey {} - Error restoring data from path {}: {}. Creating new",
                        pubkey.to_string(),
                        data_path.display().to_string(),
                        err
                    );
                }
            };
        }

        let store = Arc::new(RwLock::new(store));
        let generic_store: Arc<RwLock<dyn SerdeGenericTrait + 'static>> = store.clone();
        let persister = Persister::new(generic_store, data_path);
        persister.queue();

        let comms_data = Self { store, persister };
        Ok(comms_data)
    }

    fn restore(data_path: impl AsRef<Path>) -> Result<CommsDataStore, N3xbError> {
        let json = Persister::restore(&data_path)?;
        debug!(
            "Restored JSON from path: {} - {}",
            data_path.as_ref().display().to_string(),
            &json
        );
        let store: CommsDataStore = serde_json::from_str(&json)?;
        Ok(store)
    }

    fn setup_data_path(
        data_dir_path: impl AsRef<Path>,
        pubkey: XOnlyPublicKey,
        trade_engine_name: impl AsRef<str>,
        network: impl Borrow<BitcoinNetwork>,
    ) -> Result<PathBuf, N3xbError> {
        let dir_path = data_dir_path.as_ref().join(format!(
            "{}/{}/{}",
            pubkey.to_string(),
            trade_engine_name.as_ref(),
            network.borrow().to_string().to_lowercase()
        ));
        std::fs::create_dir_all(&dir_path)?;
        let data_path = dir_path.join("comms.json");
        Ok(data_path)
    }

    fn read_store(&self) -> RwLockReadGuard<'_, CommsDataStore> {
        match self.store.read() {
            Ok(store) => store,
            Err(error) => {
                panic!("Error reading store - {}", error);
            }
        }
    }

    fn write_store(&self) -> RwLockWriteGuard<'_, CommsDataStore> {
        match self.store.write() {
            Ok(store) => store,
            Err(error) => {
                panic!("Error writing store - {}", error);
            }
        }
    }

    pub(crate) fn relays(&self) -> Vec<(url::Url, Option<SocketAddr>)> {
        let relays = self.read_store().relays.clone();
        relays.into_iter().collect()
    }

    pub(crate) fn add_relays(&self, relays: Vec<(url::Url, Option<SocketAddr>)>) {
        let mut store = self.write_store();
        for (url, addr) in relays {
            store.relays.insert(url, addr);
        }
        self.persister.queue();
    }

    pub(crate) fn remove_relay(&self, url: &url::Url) {
        let mut store = self.write_store();
        store.relays.remove(url);
        self.persister.queue();
    }

    pub(crate) fn event_id_seen(&self, event_id: impl Into<String>) -> bool {
        self.read_store().event_ids.contains(&event_id.into())
    }

    pub(crate) fn store_event_id(&self, event_id: impl Into<String>) {
        let mut store = self.write_store();
        store.event_ids.insert(event_id.into());
        self.persister.queue();
    }

    pub(crate) fn terminate(self) {
        self.persister.terminate();
    }
}
