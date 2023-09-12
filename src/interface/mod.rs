mod maker_order_note;
mod nostr_interface;
mod peer_messaging;
mod poller;

pub mod nostr;

pub use nostr_interface::NostrInterface;

use std::sync::{Arc, Mutex};
pub type ArcInterface = Arc<Mutex<NostrInterface>>; // We might not need two layers of locking. Leaving this in for now. Potentially removing later
