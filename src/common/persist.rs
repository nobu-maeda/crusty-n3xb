use ::log::{error, trace};
use log::debug;
use std::{
    fs,
    path::Path,
    sync::{
        mpsc::{self, TrySendError},
        Arc, RwLock, RwLockReadGuard,
    },
};

use crate::common::{error::N3xbError, types::SerdeGenericTrait};

enum PersisterMsg {
    Persist,
    Close,
}

pub(crate) struct Persister {
    persist_tx: mpsc::SyncSender<PersisterMsg>,
    task_handle: std::thread::JoinHandle<()>,
}

impl Persister {
    pub(crate) fn restore(data_path: impl AsRef<Path>) -> Result<String, N3xbError> {
        let json: String = std::fs::read_to_string(data_path.as_ref())?;
        Ok(json)
    }

    pub(crate) fn new(
        store: Arc<RwLock<dyn SerdeGenericTrait>>,
        data_path: impl AsRef<Path>,
    ) -> Self {
        let (persist_tx, task_handle) = Self::setup_persistence(store, data_path);

        Self {
            persist_tx,
            task_handle,
        }
    }

    fn setup_persistence(
        store: Arc<RwLock<dyn SerdeGenericTrait>>,
        data_path: impl AsRef<Path>,
    ) -> (mpsc::SyncSender<PersisterMsg>, std::thread::JoinHandle<()>) {
        let data_path_buf = data_path.as_ref().to_path_buf();

        let (persist_tx, persist_rx) = mpsc::sync_channel(1);
        let task_handle = std::thread::spawn(move || {
            let data_path = data_path_buf.clone();
            loop {
                match persist_rx.recv() {
                    Ok(msg) => match msg {
                        PersisterMsg::Persist => {
                            let store = match store.read() {
                                Ok(store) => store,
                                Err(error) => {
                                    error!("Error reading store - {}", error);
                                    continue;
                                }
                            };
                            if let Some(error) = Self::persist(store, &data_path).err() {
                                error!(
                                    "Error persisting data to path {} - {}",
                                    data_path.display().to_string(),
                                    error
                                );
                            }
                        }
                        PersisterMsg::Close => {
                            break;
                        }
                    },
                    Err(err) => {
                        error!("Persistance channel recv Error - {}", err);
                        break;
                    }
                }
            }
            debug!(
                "Persistence thread for {} exiting",
                data_path.display().to_string()
            );
        });
        (persist_tx, task_handle)
    }

    fn persist(
        store: RwLockReadGuard<'_, dyn SerdeGenericTrait>,
        data_path: impl AsRef<Path>,
    ) -> Result<(), N3xbError> {
        let json = serde_json::to_string(&*store)?;
        let contains_type = json.contains("type");
        let contains_type_string = if contains_type {
            "containing type"
        } else {
            "not containing type"
        };

        debug!(
            "Persisting JSON {} to path: {} - {}",
            contains_type_string,
            data_path.as_ref().display().to_string(),
            json
        );

        assert!(contains_type);
        fs::write(data_path.as_ref(), json)?;
        Ok(())
    }

    pub(crate) fn queue(&self) {
        match self.persist_tx.try_send(PersisterMsg::Persist) {
            Ok(_) => {}
            Err(error) => match error {
                TrySendError::Full(_) => {
                    trace!("Persistence channel full")
                }
                TrySendError::Disconnected(_) => {
                    error!("Persistence channel disconnected")
                }
            },
        }
    }

    pub(crate) fn terminate(self) {
        self.persist_tx.send(PersisterMsg::Close).unwrap();
        if let Some(error) = self.task_handle.join().err() {
            error!("Error terminating persistence thread - {:?}", error);
        }
    }
}
