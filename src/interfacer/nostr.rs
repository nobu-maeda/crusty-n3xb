#[cfg(test)]
use tokio::sync::broadcast::Receiver;

#[cfg(not(test))]
pub use nostr_sdk::prelude::*;

#[cfg(test)]
use mockall::*;

#[cfg(test)]
pub use nostr_sdk::nostr::prelude::*;

pub use nostr_sdk::{
    client::Error,
    relay::RelayPoolNotification,
    secp256k1::XOnlyPublicKey,
    Relay,
    {
        Event, EventBuilder, EventId, Filter, Keys, Kind, Options, RelayMessage, Tag, TagKind,
        Timestamp,
    },
};

#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
use std::net::SocketAddr;

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
mock! {
    pub Client {
        pub fn with_opts(keys: &Keys, opts: Options) -> Self;
        pub fn keys(&self) -> Keys;
        pub async fn add_relay<S>(&self, url: S, proxy: Option<SocketAddr>) -> Result<(), Error> where S: Into<String> + 'static;
        pub async fn add_relays<S>(&self, relays: Vec<(S, Option<SocketAddr>)>) -> Result<(), Error> where S: Into<String> + 'static;
        pub async fn remove_relay<S>(&self, url: S) -> Result<(), Error> where S: Into<String> + 'static;
        pub async fn relays(&self) -> HashMap<Url, Relay>;
        pub async fn connect(&self);
        pub async fn send_event(&self, event: Event) -> Result<EventId, Error>;
        pub async fn get_events_of(&self, filters: Vec<Filter>, timeout: Option<Duration>) -> Result<Vec<Event>, Error>;
        pub async fn send_direct_msg<S>(&self, receiver: XOnlyPublicKey, msg: S) -> Result<EventId, Error> where S: Into<String> + 'static;
        pub fn notifications(&self) -> Receiver<RelayPoolNotification>;
        pub async fn subscribe(&self, filters: Vec<Filter>);
    }
}

#[cfg(test)]
pub use MockClient as Client;
