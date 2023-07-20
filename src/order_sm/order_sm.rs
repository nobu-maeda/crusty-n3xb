// State Machines should have 3 sources of inputs, and 3 sources of outputs, from the Trade Engine and from Nostr, as follows:
//  Trade Engine:
//   Input - Functional Commands from Trade Engine
//   Output - Notifications & Callbacks back from Trade Engine
//  Nostr:
//   Input - Peer Messages from Nostr
//   Input - Event Subscription from Nostr
//   Output - Peer Messaging to Nostr
//   Output - Event Notes publishing & updates to Nostr

pub trait OrderSM {
    // Common Order properties
    fn identifier(&self) -> String;

    // Commands common to all orders
    fn message(&self);
    fn remove(&self);
    fn complete(&self);
}

enum TradeStatus {}
