mod maker_tester;
mod relay;
mod taker_tester;

use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(test)]
mod make_order_tests {
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

        let mut manager: Manager = Manager::new(&test_engine_name).await;

        let relays = vec![(format!("{}:{}", "ws://localhost", relay.port), None)];
        manager.add_relays(relays, true).await.unwrap();

        let mut builder: OrderBuilder = OrderBuilder::new();
        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());
        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let order = builder.build().unwrap();
        manager.make_new_order(order).await.unwrap();

        let orders = loop {
            let orders = manager.query_order_notes().await.unwrap();
            if !orders.is_empty() {
                break orders;
            }
        };

        assert_eq!(orders[0].trade_uuid, SomeTestOrderParams::some_uuid());
        assert_eq!(
            orders[0].maker_obligation.kinds,
            SomeTestOrderParams::maker_obligation_kinds()
        );
        assert_eq!(
            orders[0].maker_obligation.content,
            SomeTestOrderParams::maker_obligation_content()
        );
        assert_eq!(
            orders[0].taker_obligation.kinds,
            SomeTestOrderParams::taker_obligation_kinds()
        );
        assert_eq!(
            orders[0].taker_obligation.content,
            SomeTestOrderParams::taker_obligation_content()
        );
        assert_eq!(
            orders[0].trade_details.parameters,
            SomeTestOrderParams::trade_parameters()
        );
        assert_eq!(
            orders[0].trade_details.content,
            SomeTestOrderParams::trade_details_content()
        );

        let trade_engine_specifics = orders[0]
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineMakerOrderSpecifics>()
            .unwrap();

        assert_eq!(
            trade_engine_specifics.test_specific_field,
            SomeTestParams::engine_specific_str()
        );
        assert_eq!(
            orders[0].pow_difficulty,
            SomeTestOrderParams::pow_difficulty()
        );

        // Shutdown the relay
        relay.shutdown_tx.send(()).unwrap();

        // Wait for relay to shutdown
        let thread_join = relay.handle.join();
        assert!(thread_join.is_ok());
        // assert that port is now available.
        assert!(relay::port_is_available(relay.port));
    }
}

#[cfg(test)]
mod maker_taker_flow_tests {
    use std::time::Duration;

    use super::relay;
    use super::INIT;
    use crusty_n3xb::order::Order;
    use crusty_n3xb::testing::*;
    use crusty_n3xb::{
        manager::Manager,
        order::{MakerObligation, OrderBuilder, TakerObligation, TradeDetails},
    };
    use log::info;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_order_offer_response() {
        let mut logger = env_logger::builder();
        logger.is_test(true);
        logger.filter_level(log::LevelFilter::Debug);

        INIT.call_once(|| {
            _ = logger.try_init();
        });

        let relay = relay::start_relay().unwrap();
        relay::wait_for_healthy_relay(&relay).await.unwrap();

        let test_engine_name = SomeTestParams::engine_name_str();

        let maker: Manager = Manager::new(&test_engine_name).await;

        let maker_pubkey = maker.pubkey().await;
        info!("Created Maker Manager with Pubkey {}", maker_pubkey);

        let mut taker: Manager = Manager::new(&test_engine_name).await;

        let taker_pubkey = taker.pubkey().await;
        info!("Created Taker Manager with Pubkey {}", taker_pubkey);

        let relays = vec![(format!("{}:{}", "ws://localhost", relay.port), None)];
        maker.add_relays(relays.clone(), true).await.unwrap();
        taker.add_relays(relays, true).await.unwrap();

        // Build and send the Maker Order
        let mut builder: OrderBuilder = OrderBuilder::new();
        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());
        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let order = builder.build().unwrap();
        let _ = maker.make_new_order(order).await.unwrap();

        let orders = loop {
            let orders = taker.query_order_notes().await.unwrap();
            if !orders.is_empty() {
                break orders;
            }
        };

        let mut opt_order: Option<Order> = None;
        for order in orders {
            if order.pubkey == maker_pubkey && order.trade_uuid == SomeTestOrderParams::some_uuid()
            {
                opt_order = Some(order);
            }
        }

        assert!(opt_order.is_some());
        let order = opt_order.unwrap();

        assert_eq!(order.trade_uuid, SomeTestOrderParams::some_uuid());
        assert_eq!(
            order.maker_obligation.kinds,
            SomeTestOrderParams::maker_obligation_kinds()
        );
        assert_eq!(
            order.maker_obligation.content,
            SomeTestOrderParams::maker_obligation_content()
        );
        assert_eq!(
            order.taker_obligation.kinds,
            SomeTestOrderParams::taker_obligation_kinds()
        );
        assert_eq!(
            order.taker_obligation.content,
            SomeTestOrderParams::taker_obligation_content()
        );
        assert_eq!(
            order.trade_details.parameters,
            SomeTestOrderParams::trade_parameters()
        );
        assert_eq!(
            order.trade_details.content,
            SomeTestOrderParams::trade_details_content()
        );

        let trade_engine_specifics = order
            .trade_engine_specifics
            .any_ref()
            .downcast_ref::<SomeTradeEngineMakerOrderSpecifics>()
            .unwrap();

        assert_eq!(
            trade_engine_specifics.test_specific_field,
            SomeTestParams::engine_specific_str()
        );

        assert_eq!(order.pow_difficulty, SomeTestOrderParams::pow_difficulty());

        // Create Taker Offer to take the Order
        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        taker.take_order(order, offer).await.unwrap();

        sleep(Duration::from_millis(500)).await;

        // Shutdown the relay
        relay.shutdown_tx.send(()).unwrap();

        // Wait for relay to shutdown
        let thread_join = relay.handle.join();
        assert!(thread_join.is_ok());
        // assert that port is now available.
        assert!(relay::port_is_available(relay.port));
    }

    // TODO： Dual Threaded Maker Taker Complete Flow
    //
    // Thread 1
    // - Creates Manager
    // - Create Order
    // - Make Order -> creates Maker
    // - Wait for Taker Offer Notif -> Query Offers -> Accept Offer
    // - Exit Loop on Success

    // Thread 2
    // - Creates Manager
    // - Query & poll for Orders
    // - * Optionally create ability to subscribe to a certain filter of Orders
    // - Take Order -> creates Taker
    // - Wait for Offer Acceptance Notif
    // - Exit Loop on Success

    #[tokio::test]
    async fn test_dual_thread_full_flow() {}
}
