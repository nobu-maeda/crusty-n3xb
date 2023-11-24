mod common;

#[cfg(test)]
mod integration_tests {
    use std::net::SocketAddr;
    use std::time::Duration;

    use tokio::time::sleep;

    use crusty_n3xb::manager::Manager;
    use crusty_n3xb::order::FilterTag;
    use crusty_n3xb::testing::*;

    use super::common::maker_testers::MakerTester;
    use super::common::relay::Relay;
    use super::common::taker_testers::TakerTester;

    #[tokio::test]
    async fn test_simple_four_relays_flow() {
        // setup tracing
        let _trace_sub = tracing_subscriber::fmt::try_init();

        let mut relays: Vec<Relay> = Vec::new();

        let relay: Relay = Relay::start();
        relay.wait_for_healthy_relay().await.unwrap();
        relays.push(relay);

        let relay: Relay = Relay::start();
        relay.wait_for_healthy_relay().await.unwrap();
        relays.push(relay);

        let relay: Relay = Relay::start();
        relay.wait_for_healthy_relay().await.unwrap();
        relays.push(relay);

        let relay: Relay = Relay::start();
        relay.wait_for_healthy_relay().await.unwrap();
        relays.push(relay);

        let test_engine_name = SomeTestParams::engine_name_str();
        let maker_manager = Manager::new(&test_engine_name).await;
        let taker_manager = Manager::new(&test_engine_name).await;

        let mut relay_addrs: Vec<(String, Option<SocketAddr>)> = Vec::new();

        for relay in relays.iter_mut() {
            relay_addrs.push((format!("{}:{}", "ws://localhost", relay.port), None));
        }

        maker_manager
            .add_relays(relay_addrs.clone(), true)
            .await
            .unwrap();
        taker_manager.add_relays(relay_addrs, true).await.unwrap();

        let order = SomeTestOrderParams::default_builder().build().unwrap();
        let trade_uuid = order.trade_uuid.clone();

        let maker_tester = MakerTester::start(maker_manager, order, true).await;

        sleep(Duration::from_secs(1)).await; // Wait for all 4 relays to have the order

        let mut query_filter = Vec::new();
        query_filter.push(FilterTag::TradeUuid(trade_uuid.clone()));
        query_filter.push(FilterTag::MakerObligations(
            SomeTestOrderParams::obligation_fiat_cny_kinds(),
        ));
        query_filter.push(FilterTag::TakerObligations(
            SomeTestOrderParams::obligation_bitcoin_lightning_kinds(),
        ));
        query_filter.push(FilterTag::TradeDetailParameters(
            SomeTestOrderParams::trade_parameters(),
        ));

        let taker_tester = TakerTester::start(taker_manager, query_filter, Some(trade_uuid)).await;

        maker_tester.wait_for_completion().await.unwrap();

        let order_envelopes = taker_tester.wait_for_completion().await.unwrap();
        let order_envelope = order_envelopes.first().unwrap().clone();
        assert_eq!(order_envelope.urls.len(), 4);

        relays.into_iter().for_each(|r| r.shutdown().unwrap());
    }
}
