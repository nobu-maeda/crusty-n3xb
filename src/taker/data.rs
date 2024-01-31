use std::{
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::{
        error::N3xbError,
        persist::Persister,
        types::{EventIdString, SerdeGenericTrait},
    },
    offer::Offer,
    order::OrderEnvelope,
    trade_rsp::TradeResponseEnvelope,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TakerDataStore {
    order_envelope: OrderEnvelope,
    offer: Offer,
    offer_event_id: Option<EventIdString>,
    trade_rsp_envelope: Option<TradeResponseEnvelope>,
    trade_completed: bool,
}

#[typetag::serde(name = "n3xb_taker_data")]
impl SerdeGenericTrait for TakerDataStore {
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

pub(crate) struct TakerData {
    pub(crate) trade_uuid: Uuid,
    store: Arc<RwLock<TakerDataStore>>,
    persister: Persister,
}

impl TakerData {
    pub(crate) fn new(
        dir_path: impl AsRef<Path>,
        order_envelope: OrderEnvelope,
        offer: Offer,
    ) -> Self {
        let trade_uuid = order_envelope.order.trade_uuid;
        let data_path = dir_path.as_ref().join(format!("{}-taker.json", trade_uuid));

        let store = TakerDataStore {
            order_envelope,
            offer,
            offer_event_id: None,
            trade_rsp_envelope: None,
            trade_completed: false,
        };

        let store = Arc::new(RwLock::new(store));
        let generic_store: Arc<RwLock<dyn SerdeGenericTrait + 'static>> = store.clone();
        let persister = Persister::new(generic_store, data_path);
        persister.queue();

        Self {
            trade_uuid,
            store,
            persister,
        }
    }

    pub(crate) fn restore(data_path: impl AsRef<Path>) -> Result<(Uuid, Self), N3xbError> {
        let json = Persister::restore(&data_path)?;
        let store: TakerDataStore = serde_json::from_str(&json)?;

        let trade_uuid = store.order_envelope.order.trade_uuid;

        let store = Arc::new(RwLock::new(store));
        let generic_store: Arc<RwLock<dyn SerdeGenericTrait + 'static>> = store.clone();
        let persister = Persister::new(generic_store, &data_path);
        persister.queue();

        let data = Self {
            trade_uuid,
            store,
            persister,
        };

        Ok((trade_uuid, data))
    }

    fn read_store(&self) -> RwLockReadGuard<'_, TakerDataStore> {
        match self.store.read() {
            Ok(store) => store,
            Err(error) => {
                panic!("Error reading store - {}", error);
            }
        }
    }

    fn write_store(&self) -> RwLockWriteGuard<'_, TakerDataStore> {
        match self.store.write() {
            Ok(store) => store,
            Err(error) => {
                panic!("Error writing store - {}", error);
            }
        }
    }

    // Getter methods

    pub(crate) fn order_envelope(&self) -> OrderEnvelope {
        self.read_store().order_envelope.clone()
    }

    pub(crate) fn offer(&self) -> Offer {
        self.read_store().offer.clone()
    }

    pub(crate) fn offer_event_id(&self) -> Option<EventIdString> {
        self.read_store().offer_event_id.clone()
    }

    pub(crate) fn trade_rsp_envelope(&self) -> Option<TradeResponseEnvelope> {
        self.read_store().trade_rsp_envelope.clone()
    }

    pub(crate) fn trade_completed(&self) -> bool {
        self.read_store().trade_completed
    }

    // Setter methods

    pub(crate) fn set_offer_event_id(&self, offer_event_id: EventIdString) {
        self.write_store().offer_event_id = Some(offer_event_id);
        self.persister.queue();
    }

    pub(crate) fn set_trade_rsp_envelope(&self, trade_rsp_envelope: TradeResponseEnvelope) {
        self.write_store().trade_rsp_envelope = Some(trade_rsp_envelope);
        self.persister.queue();
    }

    pub(crate) fn set_trade_completed(&self, trade_completed: bool) {
        self.write_store().trade_completed = trade_completed;
        self.persister.queue();
    }

    pub(crate) fn terminate(self) {
        self.persister.terminate()
    }
}
