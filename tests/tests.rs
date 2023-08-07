mod relay;

#[cfg(test)]
mod make_order_tests {
    use tokio::time::Duration;
    use tracing::info;

    use super::relay;
    use crusty_n3xb::order::testing::*;
    use crusty_n3xb::{
        manager::Manager,
        order::{MakerObligation, OrderBuilder, TakerObligation, TradeDetails},
    };

    #[tokio::test]
    async fn test_make_query_new_order() {
        let relay = relay::start_relay().unwrap();
        relay::wait_for_healthy_relay(&relay).await.unwrap();

        let manager: Manager<SomeTradeEngineMakerOrderSpecifics> =
            Manager::new(&SomeTestParams::engine_name_str()).await;

        let relays = vec![("ws://localhost".to_string(), relay.port, None)];
        manager.add_relays(relays).await;

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

        builder.trade_engine_specifics(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.pow_difficulty(SomeTestParams::pow_difficulty());

        let order = builder.build().unwrap();
        loop {
            match manager.make_new_order(order.clone()).await {
                Ok(_) => break,
                Err(error) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    info!("Make New Order failed. Retrying {}", error.to_string())
                }
            }
        }

        let orders = manager.query_order_notes().await.unwrap();
        assert!(orders.len() == 1);
        // TODO: Actually validate by reading back the order, Tags and JSON content is as expected

        // Shutdown the relay
        relay.shutdown_tx.send(()).unwrap();

        // Wait for relay to shutdown
        let thread_join = relay.handle.join();
        assert!(thread_join.is_ok());
        // assert that port is now available.
        assert!(relay::port_is_available(relay.port));
    }
}
