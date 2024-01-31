use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{
        error::N3xbError,
        persist::Persister,
        types::{EventIdString, SerdeGenericTrait},
    },
    offer::OfferEnvelope,
    order::Order,
    trade_rsp::TradeResponse,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MakerDataStore {
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

#[typetag::serde(name = "n3xb_maker_data")]
impl SerdeGenericTrait for MakerDataStore {
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

pub(crate) struct MakerData {
    pub(crate) trade_uuid: Uuid,
    store: Arc<RwLock<MakerDataStore>>,
    persister: Persister,
}

impl MakerData {
    pub(crate) fn new(
        dir_path: impl AsRef<Path>,
        order: Order,
        reject_invalid_offers_silently: bool,
    ) -> Self {
        let trade_uuid = order.trade_uuid;
        let data_path = dir_path.as_ref().join(format!("{}-maker.json", trade_uuid));

        let mut store = MakerDataStore {
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
        let store: MakerDataStore = serde_json::from_str(&json)?;

        let trade_uuid = store.order.trade_uuid;

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

    fn read_store(&self) -> RwLockReadGuard<'_, MakerDataStore> {
        match self.store.read() {
            Ok(store) => store,
            Err(error) => {
                panic!("Error reading store - {}", error);
            }
        }
    }

    fn write_store(&self) -> RwLockWriteGuard<'_, MakerDataStore> {
        match self.store.write() {
            Ok(store) => store,
            Err(error) => {
                panic!("Error writing store - {}", error);
            }
        }
    }

    // Getter methods

    pub(crate) fn order(&self) -> Order {
        self.read_store().order.to_owned()
    }

    pub(crate) fn relay_urls(&self) -> HashSet<Url> {
        self.read_store().relay_urls.to_owned()
    }

    pub(crate) fn order_event_id(&self) -> Option<EventIdString> {
        self.read_store().order_event_id.to_owned()
    }

    pub(crate) fn offer_envelopes(&self) -> HashMap<EventIdString, OfferEnvelope> {
        self.read_store().offer_envelopes.to_owned()
    }

    pub(crate) fn accepted_offer_event_id(&self) -> Option<EventIdString> {
        self.read_store().accepted_offer_event_id.to_owned()
    }

    pub(crate) fn trade_rsp(&self) -> Option<TradeResponse> {
        self.read_store().trade_rsp.to_owned()
    }

    pub(crate) fn trade_rsp_event_id(&self) -> Option<EventIdString> {
        self.read_store().trade_rsp_event_id.to_owned()
    }

    pub(crate) fn trade_completed(&self) -> bool {
        self.read_store().trade_completed
    }

    pub(crate) fn reject_invalid_offers_silently(&self) -> bool {
        self.read_store().reject_invalid_offers_silently.to_owned()
    }

    // Setter methods

    pub(crate) fn update_maker_order(
        &mut self,
        order_event_id: EventIdString,
        relay_urls: HashSet<Url>,
    ) {
        self.write_store().order_event_id = Some(order_event_id);
        self.write_store().relay_urls = relay_urls;
        self.persister.queue();
    }

    pub(crate) fn insert_offer_envelope(
        &mut self,
        offer_event_id: EventIdString,
        offer_envelope: OfferEnvelope,
    ) {
        self.write_store()
            .offer_envelopes
            .insert(offer_event_id, offer_envelope);
        self.persister.queue();
    }

    pub(crate) fn set_accepted_offer_event_id(&mut self, accepted_offer_event_id: EventIdString) {
        self.write_store().accepted_offer_event_id = Some(accepted_offer_event_id);
        self.persister.queue();
    }

    pub(crate) fn set_trade_rsp(
        &mut self,
        trade_rsp: TradeResponse,
        trade_rsp_event_id: EventIdString,
    ) {
        self.write_store().trade_rsp = Some(trade_rsp);
        self.write_store().trade_rsp_event_id = Some(trade_rsp_event_id);
        self.persister.queue();
    }

    pub(crate) fn set_trade_completed(&mut self, trade_completed: bool) {
        self.write_store().trade_completed = trade_completed;
        self.persister.queue();
    }

    pub(crate) fn set_reject_invalid_offers_silently(
        &mut self,
        reject_invalid_offers_silently: bool,
    ) {
        self.write_store().reject_invalid_offers_silently = reject_invalid_offers_silently;
        self.persister.queue();
    }

    pub(crate) fn terminate(self) {
        self.persister.terminate()
    }
}
