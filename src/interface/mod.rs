mod maker_order_note;
mod nostr_interface;
mod peer_messaging;

pub mod nostr;

pub use nostr_interface::NostrInterface;

use std::sync::{Arc, Mutex};
pub type ArcInterface<T, U> = Arc<Mutex<NostrInterface<T, U>>>;
