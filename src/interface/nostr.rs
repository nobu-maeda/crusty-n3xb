pub use nostr_sdk::event::Error;
pub use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, Options, Tag, TagKind};
use std::sync::{Arc, Mutex};

#[cfg(not(test))]
pub use nostr_sdk::prelude::*;

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
