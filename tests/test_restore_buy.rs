mod common;

#[cfg(test)]

mod tests {
    use std::{net::SocketAddr, str::FromStr, time::Duration};
    use tracing::error;

    use tokio::{fs, sync::mpsc, time::sleep};
    use url::Url;

    use crusty_n3xb::{
        common::error::N3xbError,
        maker::MakerNotif,
        manager::Manager,
        order::FilterTag,
        taker::TakerNotif,
        testing::{
            SomeTestOfferParams, SomeTestOrderParams, SomeTestParams, SomeTestTradeRspParams,
            TESTING_DEFAULT_CHANNEL_SIZE,
        },
        trade_rsp::TradeResponseStatus,
    };

    use super::common::{
        logger::setup as logger_setup, relay::Relay, test_trade_msgs::SomeTradeEngMsg,
    };

    #[tokio::test]
    async fn test_restore_buy() {
        // logger_setup();

        // Set up the initial state
        if let Some(error) = fs::remove_dir_all("n3xb_data/").await.err() {
            error!("Failed to remove /n3xb_data/ directory: {}", error);
        }

        let mut relays: Vec<Relay> = Vec::new();

        let relay: Relay = Relay::start();
        relay.wait_for_healthy_relay().await.unwrap();
        relays.push(relay);

        let mut relay_addrs: Vec<(Url, Option<SocketAddr>)> = Vec::new();
        for relay in relays.iter_mut() {
            let relay_addr =
                Url::from_str(&format!("{}:{}", "ws://localhost", relay.port)).unwrap();
            relay_addrs.push((relay_addr, None));
        }

        let test_engine_name = SomeTestParams::engine_name_str();
        let test_maker_private_key = SomeTestParams::maker_private_key();
        let test_taker_private_key = SomeTestParams::taker_private_key();

        // Add Relays
        {
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;
            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            maker_manager
                .add_relays(relay_addrs.clone(), true)
                .await
                .unwrap();
            taker_manager
                .add_relays(relay_addrs.clone(), true)
                .await
                .unwrap();
            maker_manager.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();
        }

        // New Maker
        {
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;
            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            // Check relays as expected
            let relays_info = maker_manager.get_relays().await;
            relays_info.iter().for_each(|relay_info| {
                assert_eq!(relay_info.url, relay_addrs[0].0);
                assert_eq!(relays_info.len(), relay_addrs.len());
            });

            let relays_info = taker_manager.get_relays().await;
            relays_info.iter().for_each(|relay_info| {
                assert_eq!(relay_info.url, relay_addrs[0].0);
                assert_eq!(relays_info.len(), relay_addrs.len());
            });

            let order = SomeTestOrderParams::default_buy_builder().build().unwrap();
            let maker = maker_manager.new_maker(order).await;
            maker.shutdown().await.unwrap();
            maker_manager.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();
        }

        let mut query_filter = Vec::new();
        query_filter.push(FilterTag::TradeUuid(SomeTestOrderParams::some_uuid()));
        query_filter.push(FilterTag::MakerObligations(
            SomeTestOrderParams::obligation_fiat_cny_kinds(),
        ));
        query_filter.push(FilterTag::TakerObligations(
            SomeTestOrderParams::obligation_bitcoin_lightning_kinds(),
        ));
        query_filter.push(FilterTag::TradeDetailParameters(
            SomeTestOrderParams::trade_parameters(),
        ));

        // Post Order
        {
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;

            // Check maker as expected
            let makers = maker_manager.get_makers().await;
            let maker = makers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (notif_tx, mut _notif_rx) =
                mpsc::channel::<Result<MakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            maker.register_notif_tx(notif_tx).await.unwrap();
            maker_manager.connect_all_relays().await.unwrap();

            maker.post_new_order().await.unwrap();
            maker.shutdown().await.unwrap();
            maker_manager.shutdown().await.unwrap();

            // Restore Taker only after Maker have sent
            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;
            taker_manager.connect_all_relays().await.unwrap();

            let order_envelopes = taker_manager.query_orders(query_filter).await.unwrap();
            assert_eq!(order_envelopes.len(), 1);

            let order_envelope = order_envelopes.first().unwrap().to_owned();
            assert_eq!(
                order_envelope.order.trade_uuid,
                SomeTestOrderParams::some_uuid()
            );

            let offer = SomeTestOfferParams::default_buy_builder().build().unwrap();

            let taker = taker_manager
                .new_taker(order_envelope, offer)
                .await
                .unwrap();
            taker.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();
        }

        // Take Order
        {
            println!("Take Order");
            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            let takers = taker_manager.get_takers().await;
            let taker = takers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (taker_notif_tx, mut _taker_notif_rx) =
                mpsc::channel::<Result<TakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            taker.register_notif_tx(taker_notif_tx).await.unwrap();
            taker_manager.connect_all_relays().await.unwrap();

            taker.take_order().await.unwrap();
            taker.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();

            sleep(Duration::from_secs(1)).await;

            // Expect Offer notify
            println!("Expect Offer notify");
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;

            let makers = maker_manager.get_makers().await;
            let maker = makers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (maker_notif_tx, mut maker_notif_rx) =
                mpsc::channel::<Result<MakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            maker.register_notif_tx(maker_notif_tx).await.unwrap();
            maker_manager.connect_all_relays().await.unwrap();

            //  Wait for Offer notifications - This can be made into a loop if wanted, or to wait for a particular offer
            let notif_result = maker_notif_rx.recv().await.unwrap();
            let _ = match notif_result.unwrap() {
                MakerNotif::Offer(offer_envelope) => offer_envelope,
                _ => panic!("Maker only expects Offer notification at this point"),
            };

            // Query Offer
            println!("Query Offer");
            let offer_envelopes = maker.query_offers().await;
            assert!(offer_envelopes.len() >= 1);
            let offer_envelope = offer_envelopes.values().next().unwrap().to_owned();

            let offer = maker
                .query_offer(offer_envelope.event_id.clone())
                .await
                .unwrap()
                .offer;
            let order = SomeTestOrderParams::default_buy_builder().build().unwrap();
            offer.validate_against(&order).unwrap();

            SomeTestOfferParams::check(
                &offer,
                &SomeTestOfferParams::default_buy_builder().build().unwrap(),
            );
            maker.shutdown().await.unwrap();
            maker_manager.shutdown().await.unwrap();
        }

        // Accept Offer
        {
            println!("Accept Offer - Restore Maker");
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;

            let makers = maker_manager.get_makers().await;
            let maker = makers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            // Should find Offer again
            let (maker_notif_tx, mut _maker_notif_rx) =
                mpsc::channel::<Result<MakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            maker.register_notif_tx(maker_notif_tx).await.unwrap();
            maker_manager.connect_all_relays().await.unwrap();

            let offer_envelopes = maker.query_offers().await;
            assert!(offer_envelopes.len() >= 1);
            let offer_envelope = offer_envelopes.values().next().unwrap().to_owned();

            let offer = maker
                .query_offer(offer_envelope.event_id.clone())
                .await
                .unwrap()
                .offer;
            let order = SomeTestOrderParams::default_buy_builder().build().unwrap();
            offer.validate_against(&order).unwrap();

            SomeTestOfferParams::check(
                &offer,
                &SomeTestOfferParams::default_buy_builder().build().unwrap(),
            );

            // Accept Offer
            println!("Accept Offer - Trade Response");
            let mut trade_rsp_builder = SomeTestTradeRspParams::default_builder();
            trade_rsp_builder.offer_event_id(offer_envelope.event_id);
            let trade_rsp = trade_rsp_builder.build().unwrap();
            maker.accept_offer(trade_rsp).await.unwrap();

            maker.shutdown().await.unwrap();
            maker_manager.shutdown().await.unwrap();

            // Expect Trade notify
            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            let takers = taker_manager.get_takers().await;
            let taker = takers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (taker_notif_tx, mut taker_notif_rx) =
                mpsc::channel::<Result<TakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            taker.register_notif_tx(taker_notif_tx).await.unwrap();
            taker_manager.connect_all_relays().await.unwrap();

            // Wait for Trade Response notifications
            let notif_result = taker_notif_rx.recv().await.unwrap();
            let trade_rsp_envelope = match notif_result.unwrap() {
                TakerNotif::TradeRsp(trade_rsp_envelope) => trade_rsp_envelope,
                _ => panic!("Taker only expects Trade Response notification at this point"),
            };

            let mut expected_trade_rsp_builder = SomeTestTradeRspParams::default_builder();
            expected_trade_rsp_builder.offer_event_id("".to_string());

            let expected_trade_rsp = expected_trade_rsp_builder.build().unwrap();
            SomeTestTradeRspParams::check(&trade_rsp_envelope.trade_rsp, &expected_trade_rsp);

            assert_eq!(
                trade_rsp_envelope.trade_rsp.trade_response,
                TradeResponseStatus::Accepted
            );

            taker.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();
        }

        // Peer Message
        {
            // Should find Trade notify again
            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            let takers = taker_manager.get_takers().await;
            let taker = takers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (taker_notif_tx, mut _taker_notif_rx) =
                mpsc::channel::<Result<TakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            taker.register_notif_tx(taker_notif_tx).await.unwrap();
            taker_manager.connect_all_relays().await.unwrap();

            let trade_rsp_envelope = taker.query_trade_rsp().await.unwrap().unwrap();

            let mut expected_trade_rsp_builder = SomeTestTradeRspParams::default_builder();
            expected_trade_rsp_builder.offer_event_id("".to_string());

            let expected_trade_rsp = expected_trade_rsp_builder.build().unwrap();
            SomeTestTradeRspParams::check(&trade_rsp_envelope.trade_rsp, &expected_trade_rsp);

            assert_eq!(
                trade_rsp_envelope.trade_rsp.trade_response,
                TradeResponseStatus::Accepted
            );

            // Send a Trade Engine specific Peer Message
            let some_trade_eng_msg = SomeTradeEngMsg {
                some_trade_specific_field: SomeTradeEngMsg::some_trade_specific_string(),
            };

            taker
                .send_peer_message(Box::new(some_trade_eng_msg))
                .await
                .unwrap();

            taker.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();

            // Expect Peer Message notify
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;

            let makers = maker_manager.get_makers().await;
            let maker = makers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (maker_notif_tx, mut maker_notif_rx) =
                mpsc::channel::<Result<MakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            maker.register_notif_tx(maker_notif_tx).await.unwrap();
            maker_manager.connect_all_relays().await.unwrap();

            let notif_result = maker_notif_rx.recv().await.unwrap();
            let peer_envelope = match notif_result.unwrap() {
                MakerNotif::Peer(peer_envelope) => peer_envelope,
                _ => panic!("Maker only expects Peer notification at this point"),
            };

            // Check Peer Message that its SomeTradeEngSpeicficMsg
            let some_trade_eng_msg = peer_envelope
                .message
                .downcast_ref::<SomeTradeEngMsg>()
                .unwrap();

            assert_eq!(
                some_trade_eng_msg.some_trade_specific_field,
                SomeTradeEngMsg::some_trade_specific_string()
            );

            maker.shutdown().await.unwrap();
            maker_manager.shutdown().await.unwrap();
        }

        // Trade Complete
        {
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;

            let makers = maker_manager.get_makers().await;
            let maker = makers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (maker_notif_tx, mut _maker_notif_rx) =
                mpsc::channel::<Result<MakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            maker.register_notif_tx(maker_notif_tx).await.unwrap();
            maker_manager.connect_all_relays().await.unwrap();
            maker.trade_complete().await.unwrap();
            maker_manager.shutdown().await.unwrap();

            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            let takers = taker_manager.get_takers().await;
            let taker = takers.get(&SomeTestOrderParams::some_uuid()).unwrap();

            let (taker_notif_tx, mut _taker_notif_rx) =
                mpsc::channel::<Result<TakerNotif, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
            taker.register_notif_tx(taker_notif_tx).await.unwrap();
            taker_manager.connect_all_relays().await.unwrap();
            taker.trade_complete().await.unwrap();
            taker_manager.shutdown().await.unwrap();
        }

        // Completed Status
        {
            // Should find Completed Trade
            let maker_manager =
                Manager::new_with_key(test_maker_private_key, &test_engine_name, "").await;

            let makers = maker_manager.get_makers().await;
            let maker = makers.get(&SomeTestOrderParams::some_uuid()).unwrap();
            assert!(maker.trade_complete().await.is_err());

            let taker_manager =
                Manager::new_with_key(test_taker_private_key, &test_engine_name, "").await;

            let takers = taker_manager.get_takers().await;
            let taker = takers.get(&SomeTestOrderParams::some_uuid()).unwrap();
            assert!(taker.trade_complete().await.is_err());

            maker.shutdown().await.unwrap();
            maker_manager.shutdown().await.unwrap();

            taker.shutdown().await.unwrap();
            taker_manager.shutdown().await.unwrap();
        }

        relays.into_iter().for_each(|r| r.shutdown().unwrap());
    }
}
