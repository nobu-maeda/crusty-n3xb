mod maker_order_note;
mod nostr_interface;
mod peer_messaging;

pub mod nostr;

pub use nostr_interface::NostrInterface;

use std::sync::{Arc, Mutex};
pub type ArcInterface<T, U> = Arc<Mutex<NostrInterface<T, U>>>; // We might not need two layers of locking. Leaving this in for now. Potentially removing later
