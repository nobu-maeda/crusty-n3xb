mod maker;
mod taker;

pub use maker::{
    maker_order::MakerOrder, maker_order_builder::MakerOrderBuilder,
    trade_engine_details::TradeEngineSpecfiicsTrait,
};

pub trait Order {
    // Common Order properties
    fn identifier(&self) -> String;

    // Commands common to all orders
    fn message(&self);
    fn remove(&self);
    fn complete(&self);
}

enum TradeStatus {}
