mod maker_order_note;
pub mod nostr;
mod nostr_interface;
mod peer_messaging;
mod take_order;
mod trade_response;

pub use nostr_interface::NostrInterface;
use std::sync::{Arc, Mutex};

pub type ArcInterface<T> = Arc<Mutex<NostrInterface<T>>>;
