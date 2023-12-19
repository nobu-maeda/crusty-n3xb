use log::error;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use url::Url;

use crate::{
    common::{error::N3xbError, types::EventIdString},
    offer::OfferEnvelope,
    order::Order,
    trade_rsp::TradeResponse,
};

#[derive(Serialize, Deserialize)]
struct MakerActorDataStore {
    order: Order,
    relay_urls: HashSet<Url>,
    order_event_id: Option<EventIdString>,
    offer_envelopes: HashMap<EventIdString, OfferEnvelope>,
    accepted_offer_event_id: Option<EventIdString>,
    trade_rsp: Option<TradeResponse>,
    trade_rsp_event_id: Option<EventIdString>,
    reject_invalid_offers_silently: bool,
}

impl MakerActorDataStore {
    // TODO: Optional - Encrypt with private key before persisting data
    async fn persist(&self) -> Result<(), N3xbError> {
        let data_json = serde_json::to_string(&self)?;
        let data_path = format!("data/maker/{}.json", self.order.trade_uuid);
        tokio::fs::write(data_path, data_json).await?;

        Ok(())
    }
}

pub(crate) struct MakerActorData {
    store: MakerActorDataStore,
}

impl MakerActorData {
    pub(crate) fn new(
        order: Order,
        relay_urls: HashSet<Url>,
        order_event_id: Option<EventIdString>,
        offer_envelopes: HashMap<EventIdString, OfferEnvelope>,
        accepted_offer_event_id: Option<EventIdString>,
        trade_rsp: Option<TradeResponse>,
        trade_rsp_event_id: Option<EventIdString>,
        reject_invalid_offers_silently: bool,
    ) -> Self {
        let store = MakerActorDataStore {
            order,
            relay_urls,
            order_event_id,
            offer_envelopes,
            accepted_offer_event_id,
            trade_rsp,
            trade_rsp_event_id,
            reject_invalid_offers_silently,
        };
        Self { store }
    }

    pub(crate) fn order(&self) -> &Order {
        &self.store.order
    }

    pub(crate) fn relay_urls(&self) -> &HashSet<Url> {
        &self.store.relay_urls
    }

    pub(crate) fn order_event_id(&self) -> &Option<EventIdString> {
        &self.store.order_event_id
    }

    pub(crate) fn offer_envelopes(&self) -> &HashMap<EventIdString, OfferEnvelope> {
        &self.store.offer_envelopes
    }

    pub(crate) fn accepted_offer_event_id(&self) -> &Option<EventIdString> {
        &self.store.accepted_offer_event_id
    }

    pub(crate) fn trade_rsp(&self) -> &Option<TradeResponse> {
        &self.store.trade_rsp
    }

    pub(crate) fn trade_rsp_event_id(&self) -> &Option<EventIdString> {
        &self.store.trade_rsp_event_id
    }

    pub(crate) fn reject_invalid_offers_silently(&self) -> bool {
        self.store.reject_invalid_offers_silently
    }

    // Setter methods

    pub(crate) fn update_maker_order(
        &mut self,
        order_event_id: EventIdString,
        relay_urls: HashSet<Url>,
    ) {
        self.store.order_event_id = Some(order_event_id);
        self.store.relay_urls = relay_urls;
    }

    pub(crate) fn set_offer_envelopes(
        &mut self,
        offer_envelopes: HashMap<EventIdString, OfferEnvelope>,
    ) {
        self.store.offer_envelopes = offer_envelopes;
    }

    pub(crate) fn set_accepted_offer_event_id(&mut self, accepted_offer_event_id: EventIdString) {
        self.store.accepted_offer_event_id = Some(accepted_offer_event_id);
    }

    pub(crate) fn set_trade_rsp(
        &mut self,
        trade_rsp: TradeResponse,
        trade_rsp_event_id: EventIdString,
    ) {
        self.store.trade_rsp = Some(trade_rsp);
        self.store.trade_rsp_event_id = Some(trade_rsp_event_id);
    }

    pub(crate) fn set_reject_invalid_offers_silently(
        &mut self,
        reject_invalid_offers_silently: bool,
    ) {
        self.store.reject_invalid_offers_silently = reject_invalid_offers_silently;
    }

    pub(crate) fn insert_offer_envelope(
        &mut self,
        offer_event_id: EventIdString,
        offer_envelope: OfferEnvelope,
    ) {
        self.store
            .offer_envelopes
            .insert(offer_event_id, offer_envelope);
    }
}
