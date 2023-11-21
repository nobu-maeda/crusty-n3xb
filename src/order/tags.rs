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
}

impl FilterTag {
    pub(crate) fn to_order_tag(self) -> OrderTag {
        match self {
            Self::MakerObligations(kinds) => OrderTag::MakerObligations(kinds.clone()),
            Self::TakerObligations(kinds) => OrderTag::TakerObligations(kinds.clone()),
            Self::TradeDetailParameters(parameters) => {
                OrderTag::TradeDetailParameters(parameters.clone())
            }
        }
    }
}

pub(crate) static N3XB_APPLICATION_TAG: &str = "n3xb";

#[derive(Clone, Debug, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
pub(crate) enum EventKind {
    MakerOrder,
}

#[derive(Clone, Debug, PartialEq)]
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
        filter_tags: Vec<FilterTag>,
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

#[cfg(test)]
mod tests {
    use crate::order::{EventKind, FilterTag, OrderTag, TradeDetails};
    use crate::testing::{SomeTestOrderParams, SomeTestParams};

    use super::N3XB_APPLICATION_TAG;

    #[tokio::test]
    async fn test_key_for_trade_uuid() {
        let uuid = SomeTestOrderParams::some_uuid();
        let order_tag = OrderTag::TradeUUID(uuid);
        let key = order_tag.key();
        assert_eq!(key, 'i');
    }

    #[tokio::test]
    async fn test_key_for_maker_obligations() {
        let maker_obligation_kinds = SomeTestOrderParams::obligation_fiat_cny_kinds();
        let order_tag = OrderTag::MakerObligations(maker_obligation_kinds);
        let key = order_tag.key();
        assert_eq!(key, 'm');
    }

    #[tokio::test]
    async fn test_key_for_taker_obligations() {
        let taker_obligation_kinds = SomeTestOrderParams::obligation_bitcoin_lightning_kinds();
        let order_tag = OrderTag::TakerObligations(taker_obligation_kinds);
        let key = order_tag.key();
        assert_eq!(key, 't');
    }

    #[tokio::test]
    async fn test_key_for_trade_detail_parameters() {
        let trade_detail_parameters = SomeTestOrderParams::trade_parameters();
        let order_tag = OrderTag::TradeDetailParameters(trade_detail_parameters);
        let key = order_tag.key();
        assert_eq!(key, 'p');
    }

    #[tokio::test]
    async fn test_key_for_trade_engine_name() {
        let trade_engine_name = SomeTestParams::engine_name_str();
        let order_tag = OrderTag::TradeEngineName(trade_engine_name);
        let key = order_tag.key();
        assert_eq!(key, 'n');
    }

    #[tokio::test]
    async fn test_key_for_event_kind() {
        let event_kind = SomeTestOrderParams::event_kind();
        let order_tag = OrderTag::EventKind(event_kind);
        let key = order_tag.key();
        assert_eq!(key, 'k');
    }

    #[tokio::test]
    async fn test_key_for_application_tag() {
        let application_tag = SomeTestOrderParams::application_tag();
        let order_tag = OrderTag::ApplicationTag(application_tag);
        let key = order_tag.key();
        assert_eq!(key, 'd');
    }

    #[tokio::test]
    async fn test_order_tag_from_trade_uuid_key_value() {
        let uuid = SomeTestOrderParams::some_uuid();
        let uuid_string = uuid.to_string();
        let key = "i";
        let value = vec![uuid_string];
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(order_tag, OrderTag::TradeUUID(uuid));
    }

    #[tokio::test]
    async fn test_order_tag_from_maker_obligations_key_value() {
        let maker_obligation_kinds = SomeTestOrderParams::obligation_fiat_cny_kinds();
        let key = "m";
        let value = maker_obligation_kinds
            .iter()
            .flat_map(|kind| kind.to_tag_strings())
            .collect();
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(
            order_tag,
            OrderTag::MakerObligations(maker_obligation_kinds)
        );
    }

    #[tokio::test]
    async fn test_order_tag_from_taker_obligations_key_value() {
        let taker_obligation_kinds = SomeTestOrderParams::obligation_bitcoin_lightning_kinds();
        let key = "t";
        let value = taker_obligation_kinds
            .iter()
            .flat_map(|kind| kind.to_tag_strings())
            .collect();
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(
            order_tag,
            OrderTag::TakerObligations(taker_obligation_kinds)
        );
    }

    #[tokio::test]
    async fn test_order_tag_from_trade_detail_parameters_key_value() {
        let trade_detail_parameters = SomeTestOrderParams::trade_parameters();
        let key = "p";
        let value = TradeDetails::parameters_to_tags(trade_detail_parameters.clone())
            .into_iter()
            .collect();
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(
            order_tag,
            OrderTag::TradeDetailParameters(trade_detail_parameters)
        );
    }

    #[tokio::test]
    async fn test_order_tag_from_trade_engine_name_key_value() {
        let trade_engine_name = SomeTestParams::engine_name_str();
        let key = "n";
        let value = vec![trade_engine_name.clone()];
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(order_tag, OrderTag::TradeEngineName(trade_engine_name));
    }

    #[tokio::test]
    async fn test_order_tag_from_event_kind_key_value() {
        let event_kind = SomeTestOrderParams::event_kind();
        let key = "k";
        let value = vec![event_kind.to_string()];
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(order_tag, OrderTag::EventKind(event_kind));
    }

    #[tokio::test]
    async fn test_order_tag_from_application_tag_key_value() {
        let application_tag = SomeTestOrderParams::application_tag();
        let key = "d";
        let value = vec![application_tag.clone()];
        let order_tag = OrderTag::from_key_value(key, value).unwrap();
        assert_eq!(order_tag, OrderTag::ApplicationTag(application_tag));
    }

    #[tokio::test]
    async fn test_order_tag_from_invalid_key_value() {
        let key = "x";
        let value = vec!["some value".to_string()];
        let order_tag = OrderTag::from_key_value(key, value);
        assert!(order_tag.is_err());
    }

    #[tokio::test]
    async fn test_order_tag_from_order() {
        let order = SomeTestOrderParams::default_builder().build().unwrap();
        let trade_engine_name = SomeTestParams::engine_name_str();
        let order_tags = OrderTag::from_order(order.clone(), trade_engine_name.clone());
        assert_eq!(order_tags.len(), 7);
        assert!(order_tags.contains(&OrderTag::TradeUUID(order.trade_uuid)));
        assert!(order_tags.contains(&OrderTag::MakerObligations(order.maker_obligation.kinds)));
        assert!(order_tags.contains(&OrderTag::TakerObligations(order.taker_obligation.kinds)));
        assert!(order_tags.contains(&OrderTag::TradeDetailParameters(
            order.trade_details.parameters
        )));
        assert!(order_tags.contains(&OrderTag::TradeEngineName(trade_engine_name.to_string())));
        assert!(order_tags.contains(&OrderTag::EventKind(EventKind::MakerOrder)));
        assert!(order_tags.contains(&OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string())));
    }

    #[tokio::test]
    async fn test_order_tags_from_filter_tags() {
        let mut filter_tags: Vec<FilterTag> = Vec::new();
        filter_tags.push(FilterTag::MakerObligations(
            SomeTestOrderParams::obligation_fiat_cny_kinds(),
        ));
        filter_tags.push(FilterTag::TakerObligations(
            SomeTestOrderParams::obligation_bitcoin_lightning_kinds(),
        ));
        filter_tags.push(FilterTag::TradeDetailParameters(
            SomeTestOrderParams::trade_parameters(),
        ));
        let trade_engine_name = SomeTestParams::engine_name_str();
        let order_tags = OrderTag::from_filter_tags(filter_tags, trade_engine_name.clone());
        assert_eq!(order_tags.len(), 6);
        assert!(order_tags.contains(&OrderTag::MakerObligations(
            SomeTestOrderParams::obligation_fiat_cny_kinds()
        )));
        assert!(order_tags.contains(&OrderTag::TakerObligations(
            SomeTestOrderParams::obligation_bitcoin_lightning_kinds()
        )));
        assert!(order_tags.contains(&OrderTag::TradeDetailParameters(
            SomeTestOrderParams::trade_parameters()
        )));
        assert!(order_tags.contains(&OrderTag::TradeEngineName(trade_engine_name.to_string())));
        assert!(order_tags.contains(&OrderTag::EventKind(EventKind::MakerOrder)));
        assert!(order_tags.contains(&OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string())));
    }
}
