mod common;

#[cfg(test)]
mod integration_tests {
    use crusty_n3xb::manager::Manager;
    use crusty_n3xb::testing::*;

    use super::common::maker_simple_tester::MakerSimpleTester;
    use super::common::relay::Relay;
    use super::common::taker_simple_tester::TakerSimpleTester;

    #[tokio::test]
    async fn test_simple_full_flow() {
        // setup tracing
        let _trace_sub = tracing_subscriber::fmt::try_init();

        let relay = Relay::start();
        relay.wait_for_healthy_relay().await.unwrap();

        let test_engine_name = SomeTestParams::engine_name_str();
        let maker_manager = Manager::new(&test_engine_name).await;
        let taker_manager = Manager::new(&test_engine_name).await;

        let relays = vec![(format!("{}:{}", "ws://localhost", relay.port), None)];
        maker_manager
            .add_relays(relays.clone(), true)
            .await
            .unwrap();
        taker_manager.add_relays(relays, true).await.unwrap();

        let order = SomeTestOrderParams::default_builder().build().unwrap();
        let trade_uuid = order.trade_uuid.clone();

        let maker_tester = MakerSimpleTester::start(maker_manager, order).await;
        let taker_tester = TakerSimpleTester::start(taker_manager, trade_uuid).await;

        maker_tester.wait_for_completion().await.unwrap();
        taker_tester.wait_for_completion().await.unwrap();

        relay.shutdown().unwrap();
    }
}
