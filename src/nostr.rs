use crate::common::OrderTag;
pub use nostr_sdk::event::Error;
pub use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, Options, Tag, TagKind};
use std::sync::{Arc, Mutex};

#[cfg(not(test))]
pub use nostr_sdk::prelude::*;

pub fn create_event_tags(order_tags: Vec<OrderTag>) -> Vec<Tag> {
    order_tags
        .iter()
        .map(|event_tag| match event_tag {
            OrderTag::TradeUUID(trade_uuid_string) => Tag::Generic(
                TagKind::Custom(event_tag.key().to_string()),
                vec![trade_uuid_string.to_owned()],
            ),
            OrderTag::MakerObligations(obligations) => Tag::Generic(
                TagKind::Custom(event_tag.key()),
                obligations.to_owned().into_iter().collect(),
            ),
            OrderTag::TakerObligations(obligations) => Tag::Generic(
                TagKind::Custom(event_tag.key()),
                obligations.to_owned().into_iter().collect(),
            ),
            OrderTag::TradeDetailParameters(parameters) => Tag::Generic(
                TagKind::Custom(event_tag.key()),
                parameters.to_owned().into_iter().collect(),
            ),
            OrderTag::TradeEngineName(name) => {
                Tag::Generic(TagKind::Custom(event_tag.key()), vec![name.to_owned()])
            }
            OrderTag::EventKind(kind) => {
                Tag::Generic(TagKind::Custom(event_tag.key()), vec![kind.to_string()])
            }
            OrderTag::ApplicationTag(app_tag) => {
                Tag::Generic(TagKind::Custom(event_tag.key()), vec![app_tag.to_owned()])
            }
        })
        .collect()
}

#[cfg(test)]
use mockall::*;

#[cfg(test)]
use std::net::SocketAddr;

#[cfg(test)]
mock! {
    pub Client {
        pub fn with_opts(keys: &Keys, opts: Options) -> Self;
        pub fn keys(&self) -> Keys;
        pub async fn add_relay<S>(&self, url: S, proxy: Option<SocketAddr>) -> Result<(), Error> where S: Into<String> + 'static;
        pub async fn connect(&self);
        pub async fn send_event(&self, event: Event) -> Result<EventId, Error>;
    }
}

#[cfg(test)]
pub use MockClient as Client;

pub type ArcClient = Arc<Mutex<Client>>;
