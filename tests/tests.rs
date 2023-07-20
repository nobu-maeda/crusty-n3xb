#[cfg(test)]
mod make_order_tests {
    use crusty_n3xb::order::testing::*;
    use crusty_n3xb::{
        manager::Manager,
        order::{MakerObligation, OrderBuilder, TakerObligation, TradeDetails, TradeEngineDetails},
    };

    #[tokio::test]
    async fn make_new_order() {
        let manager: Manager<SomeTradeEngineMakerOrderSpecifics> = Manager::new().await;
        let mut builder: OrderBuilder<SomeTradeEngineMakerOrderSpecifics> = OrderBuilder::new();

        builder.trade_uuid(SomeTestParams::some_uuid_string());

        builder.maker_obligation(MakerObligation {
            kind: SomeTestParams::maker_obligation_kind(),
            content: SomeTestParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kind: SomeTestParams::taker_obligation_kind(),
            content: SomeTestParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestParams::trade_parameters(),
            content: SomeTestParams::trade_details_content(),
        });

        builder.engine_details(TradeEngineDetails {
            trade_engine_name: SomeTestParams::engine_name_str(),
            trade_engine_specifics: SomeTradeEngineMakerOrderSpecifics {
                test_specific_field: SomeTestParams::engine_specific_str(),
            },
        });

        builder.pow_difficulty(SomeTestParams::pow_difficulty());

        let order = builder.build().unwrap();
        let _ = manager.make_new_order(order).await;

        // TODO: Actually validate by reading back the order, Tags and JSON content is as expected
    }
}
