mod relay;

#[cfg(test)]
mod make_order_tests {
    use tokio::time::Duration;
    use tracing::info;

    use super::relay;
    use crusty_n3xb::testing::*;
    use crusty_n3xb::{
        manager::Manager,
        order::{MakerObligation, OrderBuilder, TakerObligation, TradeDetails},
    };

    #[tokio::test]
    async fn test_make_query_new_order() {
        let relay = relay::start_relay().unwrap();
        relay::wait_for_healthy_relay(&relay).await.unwrap();

        let test_engine_name = SomeTestParams::engine_name_str();

        let mut manager: Manager<
            SomeTradeEngineMakerOrderSpecifics,
            SomeTradeEngineTakerOfferSpecifics,
        > = Manager::new(&test_engine_name).await;

        let relays = vec![("ws://localhost".to_string(), relay.port, None)];
        manager.add_relays(relays).await;

        let mut builder: OrderBuilder<SomeTradeEngineMakerOrderSpecifics> = OrderBuilder::new();
        builder.trade_uuid(SomeTestParams::some_uuid_string());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestParams::maker_obligation_kind(),
            content: SomeTestParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestParams::taker_obligation_kind(),
            content: SomeTestParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestParams::trade_parameters(),
            content: SomeTestParams::trade_details_content(),
        });

        builder.trade_engine_specifics(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.pow_difficulty(SomeTestParams::pow_difficulty());

        let order = builder.build().unwrap();
        manager.make_new_order(order.clone()).await.unwrap();

        let orders = manager.query_order_notes().await.unwrap();
        assert_eq!(orders.len(), 1);

        assert_eq!(orders[0].trade_uuid, SomeTestParams::some_uuid_string());
        assert_eq!(
            orders[0].maker_obligation.kinds,
            SomeTestParams::maker_obligation_kind()
        );
        assert_eq!(
            orders[0].maker_obligation.content,
            SomeTestParams::maker_obligation_content()
        );
        assert_eq!(
            orders[0].taker_obligation.kinds,
            SomeTestParams::taker_obligation_kind()
        );
        assert_eq!(
            orders[0].taker_obligation.content,
            SomeTestParams::taker_obligation_content()
        );
        assert_eq!(
            orders[0].trade_details.parameters,
            SomeTestParams::trade_parameters()
        );
        assert_eq!(
            orders[0].trade_details.content,
            SomeTestParams::trade_details_content()
        );
        assert_eq!(
            orders[0].trade_engine_specifics.test_specific_field,
            SomeTestParams::engine_specific_str()
        );
        assert_eq!(orders[0].pow_difficulty, SomeTestParams::pow_difficulty());

        // Shutdown the relay
        relay.shutdown_tx.send(()).unwrap();

        // Wait for relay to shutdown
        let thread_join = relay.handle.join();
        assert!(thread_join.is_ok());
        // assert that port is now available.
        assert!(relay::port_is_available(relay.port));
    }
}
