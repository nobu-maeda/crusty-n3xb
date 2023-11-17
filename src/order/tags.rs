use std::{collections::HashSet, str::FromStr};

use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::common::{error::N3xbError, types::ObligationKind};

use super::{Order, TradeDetails, TradeParameter};

#[derive(Clone, Debug, PartialEq, Eq, Display, EnumString, IntoStaticStr)]

pub enum FilterTag {
    MakerObligations(HashSet<ObligationKind>),
    TakerObligations(HashSet<ObligationKind>),
    TradeDetailParameters(HashSet<TradeParameter>),
    TradeEngineName(String),
}

impl FilterTag {
    pub(crate) fn to_order_tag(self) -> OrderTag {
        match self {
            Self::MakerObligations(kinds) => OrderTag::MakerObligations(kinds.clone()),
            Self::TakerObligations(kinds) => OrderTag::TakerObligations(kinds.clone()),
            Self::TradeDetailParameters(parameters) => {
                OrderTag::TradeDetailParameters(parameters.clone())
            }
            Self::TradeEngineName(name) => OrderTag::TradeEngineName(name.clone()),
        }
    }
}

pub(crate) static N3XB_APPLICATION_TAG: &str = "n3xb";

#[derive(Clone, Debug, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
pub(crate) enum EventKind {
    MakerOrder,
}

#[derive(Clone, Debug)]
pub(crate) enum OrderTag {
    TradeUUID(Uuid),
    MakerObligations(HashSet<ObligationKind>),
    TakerObligations(HashSet<ObligationKind>),
    TradeDetailParameters(HashSet<TradeParameter>),
    TradeEngineName(String),
    EventKind(EventKind),
    ApplicationTag(String),
}

const ORDER_TAG_TRADE_UUID_KEY: char = 'i';
const ORDER_TAG_MAKER_OBLIGATIONS_KEY: char = 'm';
const ORDER_TAG_TAKER_OBLIGATIONS_KEY: char = 't';
const ORDER_TAG_TRADE_DETAIL_PARAMETERS_KEY: char = 'p';
const ORDER_TAG_TRADE_ENGINE_NAME_KEY: char = 'n';
const ORDER_TAG_EVENT_KIND_KEY: char = 'k';
const ORDER_TAG_APPLICATION_TAG_KEY: char = 'd';

impl OrderTag {
    pub(crate) fn key(&self) -> char {
        match self {
            OrderTag::TradeUUID(_) => ORDER_TAG_TRADE_UUID_KEY,
            OrderTag::MakerObligations(_) => ORDER_TAG_MAKER_OBLIGATIONS_KEY,
            OrderTag::TakerObligations(_) => ORDER_TAG_TAKER_OBLIGATIONS_KEY,
            OrderTag::TradeDetailParameters(_) => ORDER_TAG_TRADE_DETAIL_PARAMETERS_KEY,
            OrderTag::TradeEngineName(_) => ORDER_TAG_TRADE_ENGINE_NAME_KEY,
            OrderTag::EventKind(_) => ORDER_TAG_EVENT_KIND_KEY,
            OrderTag::ApplicationTag(_) => ORDER_TAG_APPLICATION_TAG_KEY,
        }
    }

    pub(crate) fn from_key_value(
        key: impl AsRef<str>,
        value: Vec<String>,
    ) -> Result<OrderTag, N3xbError> {
        match key.as_ref().chars().next().unwrap() {
            ORDER_TAG_TRADE_UUID_KEY => {
                let uuid_string = value[0].clone();
                match Uuid::from_str(uuid_string.as_str()) {
                    Ok(uuid) => Ok(OrderTag::TradeUUID(uuid)),
                    Err(error) => Err(N3xbError::Simple(format!(
                        "Trade UUID Order Tag does not contain valid UUID string - {}",
                        error
                    ))),
                }
            }
            ORDER_TAG_MAKER_OBLIGATIONS_KEY => {
                let tag_set: HashSet<String> = HashSet::from_iter(value);
                let kinds_set = ObligationKind::from_tag_strings(tag_set)?;
                Ok(OrderTag::MakerObligations(kinds_set))
            }
            ORDER_TAG_TAKER_OBLIGATIONS_KEY => {
                let tag_set: HashSet<String> = HashSet::from_iter(value);
                let kinds_set = ObligationKind::from_tag_strings(tag_set)?;
                Ok(OrderTag::TakerObligations(kinds_set))
            }
            ORDER_TAG_TRADE_DETAIL_PARAMETERS_KEY => {
                let tag_set: HashSet<String> = HashSet::from_iter(value);
                let parameters_set = TradeDetails::tags_to_parameters(tag_set);
                Ok(OrderTag::TradeDetailParameters(parameters_set))
            }
            ORDER_TAG_TRADE_ENGINE_NAME_KEY => Ok(OrderTag::TradeEngineName(value[0].clone())),
            ORDER_TAG_EVENT_KIND_KEY => {
                let event_kind = EventKind::from_str(value[0].as_str())?;
                Ok(OrderTag::EventKind(event_kind))
            }
            ORDER_TAG_APPLICATION_TAG_KEY => Ok(OrderTag::ApplicationTag(value[0].clone())),
            _ => Err(N3xbError::Simple(format!(
                "Unrecognized key '{}' for Order Tag",
                key.as_ref()
            ))),
        }
    }

    pub(crate) fn from_order(order: Order, trade_engine_name: impl AsRef<str>) -> Vec<OrderTag> {
        let mut order_tags: Vec<OrderTag> = Vec::new();
        order_tags.push(OrderTag::TradeUUID(order.trade_uuid));
        order_tags.push(OrderTag::MakerObligations(order.maker_obligation.kinds));
        order_tags.push(OrderTag::TakerObligations(order.taker_obligation.kinds));
        order_tags.push(OrderTag::TradeDetailParameters(
            order.trade_details.parameters,
        ));
        order_tags.push(OrderTag::TradeEngineName(
            trade_engine_name.as_ref().to_owned(),
        ));
        order_tags.push(OrderTag::EventKind(EventKind::MakerOrder));
        order_tags.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));
        order_tags
    }

    pub(crate) fn from_filter_tags(
        filter_tags: HashSet<FilterTag>,
        trade_engine_name: impl AsRef<str>,
    ) -> Vec<OrderTag> {
        let mut order_tags: Vec<OrderTag> = Vec::new();
        for filter_tag in filter_tags {
            order_tags.push(filter_tag.to_order_tag());
        }
        order_tags.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));
        order_tags.push(OrderTag::EventKind(EventKind::MakerOrder));
        order_tags.push(OrderTag::TradeEngineName(
            trade_engine_name.as_ref().to_owned(),
        ));
        order_tags
    }
}
