use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, TrySendError},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
    time::SystemTime,
};

use crate::common::{error::N3xbError, utils};
use log::{error, trace};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CommsDataStore {
    relays: HashMap<url::Url, Option<SocketAddr>>,
    // filters:
    last_event: SystemTime,
}

impl CommsDataStore {
    fn persist(&self, data_path: impl AsRef<Path>) -> Result<(), N3xbError> {
        let data_json = serde_json::to_string(&self)?;
        utils::persist(data_json, data_path)
    }

    fn restore(data_path: impl AsRef<Path>) -> Result<Self, N3xbError> {
        let comms_json = utils::restore(data_path)?;
        let comms_data: Self = serde_json::from_str(&comms_json)?;
        Ok(comms_data)
    }
}

enum CommsDataMsg {
    Persist,
    Close,
}

pub(crate) struct CommsData {
    persist_tx: mpsc::SyncSender<CommsDataMsg>,
    pubkey_string: String,
    store: Arc<RwLock<CommsDataStore>>,
    task_handle: std::thread::JoinHandle<()>,
}

impl CommsData {
    pub(crate) async fn new(
        data_dir_path: impl AsRef<Path>,
        pubkey: XOnlyPublicKey,
    ) -> Result<Self, N3xbError> {
        let data_path = Self::setup_data_path(&data_dir_path, pubkey.to_string()).await?;

        let mut store = CommsDataStore {
            relays: HashMap::new(),
            last_event: SystemTime::now(),
        };

        if data_path.exists() {
            match CommsDataStore::restore(&data_path) {
                Ok(restored_data) => {
                    store = restored_data;
                }
                Err(err) => {
                    error!(
                        "Comms w/ Pubkey {} - Error restoring data from path {}: {}",
                        pubkey.to_string(),
                        data_path.display().to_string(),
                        err
                    );
                }
            };
        }

        let store = Arc::new(RwLock::new(store));
        let (persist_tx, task_handle) =
            Self::setup_persistance(store.clone(), data_path, pubkey.to_string());
        let data = Self {
            persist_tx,
            pubkey_string: pubkey.to_string(),
            store,
            task_handle,
        };
        Ok(data)
    }

    async fn setup_data_path(
        data_dir_path: impl AsRef<Path>,
        pubkey_string: String,
    ) -> Result<PathBuf, N3xbError> {
        let dir_path = data_dir_path.as_ref().join(format!("{}/", pubkey_string));
        tokio::fs::create_dir_all(&dir_path).await?;
        let data_path = dir_path.join("comms.json");
        Ok(data_path)
    }

    fn setup_persistance(
        store: Arc<RwLock<CommsDataStore>>,
        data_path: impl AsRef<Path>,
        pubkey_string: String,
    ) -> (mpsc::SyncSender<CommsDataMsg>, std::thread::JoinHandle<()>) {
        let (persist_tx, persist_rx) = mpsc::sync_channel(1);
        let data_path_buf = data_path.as_ref().to_path_buf();

        let task_handle = std::thread::spawn(move || {
            let data_path_buf = data_path_buf.clone();
            loop {
                match persist_rx.recv() {
                    Ok(msg) => match msg {
                        CommsDataMsg::Persist => {
                            let store = match store.read() {
                                Ok(store) => store,
                                Err(error) => {
                                    error!("Error reading store - {}", error);
                                    continue;
                                }
                            };

                            if let Some(err) = store.persist(&data_path_buf).err() {
                                error!(
                                    "Comms w/ Pubkey {} - Error persisting data: {}",
                                    pubkey_string, err
                                );
                            }
                        }
                        CommsDataMsg::Close => {
                            break;
                        }
                    },
                    Err(err) => {
                        error!(
                            "Comms w/ Pubkey {} - Error receiving persistance message: {}",
                            pubkey_string, err
                        );
                        break;
                    }
                }
            }
        });
        (persist_tx, task_handle)
    }

    fn queue_persistance(&self) {
        match self.persist_tx.try_send(CommsDataMsg::Persist) {
            Ok(_) => {}
            Err(error) => match error {
                TrySendError::Full(_) => {
                    trace!(
                        "Comms w/ Pubkey {} - Persistance channel full",
                        self.pubkey_string
                    )
                }
                TrySendError::Disconnected(_) => {
                    error!(
                        "Comms w/ Pubkey {} - Persistance channel disconnected",
                        self.pubkey_string
                    )
                }
            },
        }
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

    pub(crate) async fn relays(&self) -> Vec<(url::Url, Option<SocketAddr>)> {
        let relays = self.read_store().relays.clone();
        relays.into_iter().collect()
    }

    pub(crate) async fn add_relays(&self, relays: Vec<(url::Url, Option<SocketAddr>)>) {
        let mut store = self.write_store();
        for (url, addr) in relays {
            store.relays.insert(url, addr);
        }
        self.queue_persistance();
    }

    pub(crate) async fn remove_relay(&self, url: &url::Url) {
        let mut store = self.write_store();
        store.relays.remove(url);
        self.queue_persistance();
    }

    pub(crate) async fn last_event(&self) -> SystemTime {
        self.read_store().last_event
    }

    pub(crate) async fn set_last_event(&self, last_event: SystemTime) {
        let mut store = self.write_store();
        store.last_event = last_event;
        self.queue_persistance();
    }

    pub(crate) fn terminate(self) {
        self.persist_tx.send(CommsDataMsg::Close).unwrap();
        if let Some(error) = self.task_handle.join().err() {
            error!("Error terminating persistence thread - {:?}", error);
        }
    }
}
