mod maker_order_note;
pub mod nostr;
mod nostr_interface;
mod peer_messaging;

pub use nostr_interface::NostrInterface;
use std::sync::{Arc, Mutex};

pub type ArcInterface<T> = Arc<Mutex<NostrInterface<T>>>;
