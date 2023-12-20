use std::{path::Path, sync::Arc};

use log::{error, trace};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::{
    common::{error::N3xbError, types::EventIdString, utils},
    offer::Offer,
    order::OrderEnvelope,
    trade_rsp::TradeResponseEnvelope,
};

#[derive(Serialize, Deserialize)]
struct TakerActorDataStore {
    order_envelope: OrderEnvelope,
    offer: Offer,
    offer_event_id: Option<EventIdString>,
    trade_rsp_envelope: Option<TradeResponseEnvelope>,
    trade_completed: bool,
}

impl TakerActorDataStore {
    async fn persist(&self, dir_path: impl AsRef<Path>) -> Result<(), N3xbError> {
        let data_json = serde_json::to_string(&self)?;
        let data_path = dir_path.as_ref().join(format!(
            "{}-taker.json",
            self.order_envelope.order.trade_uuid
        ));
        utils::persist(data_json, data_path).await
    }

    async fn restore(data_path: impl AsRef<Path>) -> Result<Self, N3xbError> {
        let taker_json = utils::restore(data_path).await?;
        let taker_data: Self = serde_json::from_str(&taker_json)?;
        Ok(taker_data)
    }
}

pub(crate) struct TakerActorData {
    pub(crate) trade_uuid: Uuid,
    persist_tx: mpsc::Sender<()>,
    store: Arc<RwLock<TakerActorDataStore>>,
}

impl TakerActorData {
    pub(crate) fn new(
        dir_path: impl AsRef<Path>,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Self {
        let trade_uuid = order_envelope.order.trade_uuid;
        let store = TakerActorDataStore {
            order_envelope,
            offer,
            offer_event_id: None,
            trade_rsp_envelope: None,
            trade_completed: false,
        };
        let store = Arc::new(RwLock::new(store));

        Self {
            persist_tx: Self::setup_persistance(store.clone(), trade_uuid, &dir_path),
            trade_uuid,
            store,
        }
    }

    pub(crate) async fn restore(data_path: impl AsRef<Path>) -> Result<(Uuid, Self), N3xbError> {
        let store = TakerActorDataStore::restore(&data_path).await?;
        let trade_uuid = store.order_envelope.order.trade_uuid;

        let store = Arc::new(RwLock::new(store));
        let data = Self {
            persist_tx: Self::setup_persistance(store.clone(), trade_uuid, &data_path),
            trade_uuid,
            store,
        };

        Ok((trade_uuid, data))
    }

    fn setup_persistance(
        store: Arc<RwLock<TakerActorDataStore>>,
        trade_uuid: Uuid,
        dir_path: impl AsRef<Path>,
    ) -> mpsc::Sender<()> {
        // No more than 1 persistance request is allowed nor needed.
        // This is essentilaly a debounce mechanism
        let (persist_tx, mut persist_rx) = mpsc::channel(1);
        let dir_path_buf = dir_path.as_ref().to_path_buf();

        tokio::spawn(async move {
            let dir_path_buf = dir_path_buf.clone();
            loop {
                persist_rx.recv().await;
                match store.read().await.persist(&dir_path_buf).await {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Taker w/ Trade UUID {} - Error persisting data: {}",
                            trade_uuid, err
                        );
                    }
                }
            }
        });
        persist_tx
    }

    fn queue_persistance(&self) {
        match self.persist_tx.try_send(()) {
            Ok(_) => {}
            Err(error) => match error {
                mpsc::error::TrySendError::Full(_) => {
                    trace!(
                        "Taker w/ TradeUUID {} - Persistance channel full",
                        self.trade_uuid
                    )
                }
                mpsc::error::TrySendError::Closed(_) => {
                    error!(
                        "Taker w/ TradeUUID {} - Persistance channel closed",
                        self.trade_uuid
                    )
                }
            },
        }
    }

    // Getter methods

    pub(crate) async fn order_envelope(&self) -> OrderEnvelope {
        self.store.read().await.order_envelope.clone()
    }

    pub(crate) async fn offer(&self) -> Offer {
        self.store.read().await.offer.clone()
    }

    pub(crate) async fn offer_event_id(&self) -> Option<EventIdString> {
        self.store.read().await.offer_event_id.clone()
    }

    pub(crate) async fn trade_rsp_envelope(&self) -> Option<TradeResponseEnvelope> {
        self.store.read().await.trade_rsp_envelope.clone()
    }

    pub(crate) async fn trade_completed(&self) -> bool {
        self.store.read().await.trade_completed
    }

    // Setter methods

    pub(crate) async fn set_offer_event_id(&self, offer_event_id: EventIdString) {
        self.store.write().await.offer_event_id = Some(offer_event_id);
        self.queue_persistance();
    }

    pub(crate) async fn set_trade_rsp_envelope(&self, trade_rsp_envelope: TradeResponseEnvelope) {
        self.store.write().await.trade_rsp_envelope = Some(trade_rsp_envelope);
        self.queue_persistance();
    }

    pub(crate) async fn set_trade_completed(&self, trade_completed: bool) {
        self.store.write().await.trade_completed = trade_completed;
        self.queue_persistance();
    }
}
