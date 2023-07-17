mod maker_sm;
mod taker_sm;

pub trait OrderSM {
    // Common Order properties
    fn identifier(&self) -> String;

    // Commands common to all orders
    fn message(&self);
    fn remove(&self);
    fn complete(&self);
}

enum TradeStatus {}
