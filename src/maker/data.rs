use log::{error, trace};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{mpsc, RwLock},
};
use url::Url;

use crate::{
    common::{error::N3xbError, types::EventIdString, utils},
    offer::OfferEnvelope,
    order::Order,
    trade_rsp::TradeResponse,
};

#[derive(Serialize, Deserialize)]
struct MakerActorDataStore {
    // Order state data
    order: Order,
    relay_urls: HashSet<Url>,
    order_event_id: Option<EventIdString>,
    offer_envelopes: HashMap<EventIdString, OfferEnvelope>,
    accepted_offer_event_id: Option<EventIdString>,
    trade_rsp: Option<TradeResponse>,
    trade_rsp_event_id: Option<EventIdString>,
    trade_completed: bool,

    // Order specific settings
    reject_invalid_offers_silently: bool,
}

impl MakerActorDataStore {
    async fn persist(&self, dir_path: impl AsRef<Path>) -> Result<(), N3xbError> {
        let data_json = serde_json::to_string(&self)?;
        let data_path = dir_path
            .as_ref()
            .join(format!("{}-maker.json", self.order.trade_uuid));
        utils::persist(data_json, data_path).await
    }

    async fn restore(data_path: impl AsRef<Path>) -> Result<Self, N3xbError> {
        let maker_json = utils::restore(data_path).await?;
        let maker_data: Self = serde_json::from_str(&maker_json)?;
        Ok(maker_data)
    }
}

enum MakerActorDataMsg {
    Persist,
    Close,
}

pub(crate) struct MakerActorData {
    pub(crate) trade_uuid: Uuid,
    persist_tx: mpsc::Sender<MakerActorDataMsg>,
    store: Arc<RwLock<MakerActorDataStore>>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl MakerActorData {
    pub(crate) fn new(
        dir_path: impl AsRef<Path>,
        order: Order,
        reject_invalid_offers_silently: bool,
    ) -> Self {
        let trade_uuid = order.trade_uuid;
        let store = MakerActorDataStore {
            order,
            relay_urls: HashSet::new(),
            order_event_id: None,
            offer_envelopes: HashMap::new(),
            accepted_offer_event_id: None,
            trade_rsp: None,
            trade_rsp_event_id: None,
            trade_completed: false,
            reject_invalid_offers_silently,
        };
        let store = Arc::new(RwLock::new(store));
        let (persist_tx, task_handle) =
            Self::setup_persistance(store.clone(), trade_uuid, &dir_path);

        Self {
            persist_tx,
            trade_uuid,
            store,
            task_handle,
        }
    }

    pub(crate) async fn restore(data_path: impl AsRef<Path>) -> Result<(Uuid, Self), N3xbError> {
        let store = MakerActorDataStore::restore(&data_path).await?;
        let trade_uuid = store.order.trade_uuid;

        let store = Arc::new(RwLock::new(store));
        let dir_path = data_path.as_ref().parent().unwrap();

        let (persist_tx, task_handle) =
            Self::setup_persistance(store.clone(), trade_uuid, &dir_path);

        let data = Self {
            persist_tx,
            trade_uuid,
            store,
            task_handle,
        };
        Ok((trade_uuid, data))
    }

    fn setup_persistance(
        store: Arc<RwLock<MakerActorDataStore>>,
        trade_uuid: Uuid,
        dir_path: impl AsRef<Path>,
    ) -> (mpsc::Sender<MakerActorDataMsg>, tokio::task::JoinHandle<()>) {
        // No more than 1 persistance request is allowed nor needed.
        // This is essentilaly a debounce mechanism
        let (persist_tx, mut persist_rx) = mpsc::channel(1);
        let dir_path_buf = dir_path.as_ref().to_path_buf();

        let task_handle = tokio::spawn(async move {
            let dir_path_buf = dir_path_buf.clone();
            loop {
                select! {
                    Some(msg) = persist_rx.recv() => {
                        match msg {
                            MakerActorDataMsg::Persist => {
                                if let Some(err) = store.read().await.persist(&dir_path_buf).await.err() {
                                    error!(
                                        "Maker w/ TradeUUID {} - Error persisting data: {}",
                                        trade_uuid, err
                                    );
                                }
                            }
                            MakerActorDataMsg::Close => {
                                break;
                            }
                        }

                    },
                    else => break,
                }
            }
        });
        (persist_tx, task_handle)
    }

    fn queue_persistance(&self) {
        match self.persist_tx.try_send(MakerActorDataMsg::Persist) {
            Ok(_) => {}
            Err(error) => match error {
                mpsc::error::TrySendError::Full(_) => {
                    trace!(
                        "Maker w/ TradeUUID {} - Persistance channel full",
                        self.trade_uuid
                    )
                }
                mpsc::error::TrySendError::Closed(_) => {
                    error!(
                        "Maker w/ TradeUUID {} - Persistance channel closed",
                        self.trade_uuid
                    )
                }
            },
        }
    }

    // Getter methods

    pub(crate) async fn order(&self) -> Order {
        self.store.read().await.order.to_owned()
    }

    pub(crate) async fn relay_urls(&self) -> HashSet<Url> {
        self.store.read().await.relay_urls.to_owned()
    }

    pub(crate) async fn order_event_id(&self) -> Option<EventIdString> {
        self.store.read().await.order_event_id.to_owned()
    }

    pub(crate) async fn offer_envelopes(&self) -> HashMap<EventIdString, OfferEnvelope> {
        self.store.read().await.offer_envelopes.to_owned()
    }

    pub(crate) async fn accepted_offer_event_id(&self) -> Option<EventIdString> {
        self.store.read().await.accepted_offer_event_id.to_owned()
    }

    pub(crate) async fn trade_rsp(&self) -> Option<TradeResponse> {
        self.store.read().await.trade_rsp.to_owned()
    }

    pub(crate) async fn trade_rsp_event_id(&self) -> Option<EventIdString> {
        self.store.read().await.trade_rsp_event_id.to_owned()
    }

    pub(crate) async fn trade_completed(&self) -> bool {
        self.store.read().await.trade_completed
    }

    pub(crate) async fn reject_invalid_offers_silently(&self) -> bool {
        self.store
            .read()
            .await
            .reject_invalid_offers_silently
            .to_owned()
    }

    // Setter methods

    pub(crate) async fn update_maker_order(
        &mut self,
        order_event_id: EventIdString,
        relay_urls: HashSet<Url>,
    ) {
        self.store.write().await.order_event_id = Some(order_event_id);
        self.store.write().await.relay_urls = relay_urls;
        self.queue_persistance();
    }

    pub(crate) async fn insert_offer_envelope(
        &mut self,
        offer_event_id: EventIdString,
        offer_envelope: OfferEnvelope,
    ) {
        self.store
            .write()
            .await
            .offer_envelopes
            .insert(offer_event_id, offer_envelope);
        self.queue_persistance();
    }

    pub(crate) async fn set_accepted_offer_event_id(
        &mut self,
        accepted_offer_event_id: EventIdString,
    ) {
        self.store.write().await.accepted_offer_event_id = Some(accepted_offer_event_id);
        self.queue_persistance();
    }

    pub(crate) async fn set_trade_rsp(
        &mut self,
        trade_rsp: TradeResponse,
        trade_rsp_event_id: EventIdString,
    ) {
        self.store.write().await.trade_rsp = Some(trade_rsp);
        self.store.write().await.trade_rsp_event_id = Some(trade_rsp_event_id);
        self.queue_persistance();
    }

    pub(crate) async fn set_trade_completed(&mut self, trade_completed: bool) {
        self.store.write().await.trade_completed = trade_completed;
        self.queue_persistance();
    }

    pub(crate) async fn set_reject_invalid_offers_silently(
        &mut self,
        reject_invalid_offers_silently: bool,
    ) {
        self.store.write().await.reject_invalid_offers_silently = reject_invalid_offers_silently;
        self.queue_persistance();
    }

    pub(crate) async fn terminate(self) -> Result<(), N3xbError> {
        self.persist_tx.send(MakerActorDataMsg::Close).await?;
        self.task_handle.await?;
        Ok(())
    }
}
