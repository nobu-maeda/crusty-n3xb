mod maker;
mod taker;

pub use maker::{
    maker_order::MakerOrder, maker_order_builder::MakerOrderBuilder,
    trade_engine_details::TradeEngineSpecfiicsTrait,
};

pub trait Order {
    // Common Order properties
    fn identifier() -> String;

    // Commands common to all orders
    fn message();
    fn remove();
    fn complete();
}

enum TradeStatus {}
