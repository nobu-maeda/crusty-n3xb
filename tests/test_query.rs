mod common;

#[cfg(test)]
mod test_query {
    use std::str::FromStr;
    use std::time::Duration;

    use tokio::time::sleep;
    use url::Url;
    use uuid::Uuid;

    use crusty_n3xb::manager::Manager;
    use crusty_n3xb::order::{FilterTag, MakerObligation, TakerObligation};
    use crusty_n3xb::testing::*;

    use super::common::maker_testers::MakerTester;
    use super::common::relay::Relay;
    use super::common::taker_testers::TakerTester;

    #[tokio::test]
    async fn test_query_order_filtering() {
        let relay1: Relay = Relay::start();
        relay1.wait_for_healthy_relay().await.unwrap();
        let relay1_addr = (format!("{}:{}", "ws://localhost", relay1.port), None);
        let relay1_url = Url::from_str(&relay1_addr.0).unwrap();

        let relay2: Relay = Relay::start();
        relay2.wait_for_healthy_relay().await.unwrap();
        let relay2_addr = (format!("{}:{}", "ws://localhost", relay2.port), None);
        let relay2_url = Url::from_str(&relay2_addr.0).unwrap();

        let relay3: Relay = Relay::start();
        relay3.wait_for_healthy_relay().await.unwrap();
        let relay3_addr = (format!("{}:{}", "ws://localhost", relay3.port), None);
        let _relay3_url = Url::from_str(&relay3_addr.0).unwrap();

        let relay4: Relay = Relay::start();
        relay4.wait_for_healthy_relay().await.unwrap();
        let relay4_addr = (format!("{}:{}", "ws://localhost", relay4.port), None);
        let relay4_url = Url::from_str(&relay4_addr.0).unwrap();

        let test_engine_name1 = String::from("TestEngine1");
        let test_engine_name2 = String::from("TestEngine2");

        let manager_m1 = Manager::new(&test_engine_name1).await;
        let manager_m2 = Manager::new(&test_engine_name2).await;
        let manager_m3 = Manager::new(&test_engine_name1).await;
        let manager_t1 = Manager::new(&test_engine_name1).await;
        let manager_t2 = Manager::new(&test_engine_name2).await;

        manager_m1
            .add_relays(
                vec![
                    relay1_addr.clone(),
                    relay2_addr.clone(),
                    relay3_addr.clone(),
                    relay4_addr.clone(),
                ],
                true,
            )
            .await
            .unwrap();

        manager_m2
            .add_relays(
                vec![
                    relay1_addr.clone(),
                    relay2_addr.clone(),
                    relay3_addr.clone(),
                ],
                true,
            )
            .await
            .unwrap();

        manager_m3
            .add_relays(vec![relay1_addr.clone()], true)
            .await
            .unwrap();

        manager_t1
            .add_relays(
                vec![
                    relay1_addr.clone(),
                    relay2_addr.clone(),
                    relay4_addr.clone(),
                ],
                true,
            )
            .await
            .unwrap();

        manager_t2
            .add_relays(vec![relay1_addr, relay3_addr, relay4_addr], true)
            .await
            .unwrap();

        // Create 3 orders,
        // Manager M1 posts Order 1 tracked by Maker M1A - RMB for BTC/LN
        let uuid1 = Uuid::new_v4();
        let maker_obligation1 = MakerObligation {
            kinds: SomeTestOrderParams::obligation_fiat_cny_kinds(),
            content: SomeTestOrderParams::maker_obligation_fiat_cny_content(),
        };
        let taker_obligation1 = TakerObligation {
            kinds: SomeTestOrderParams::obligation_bitcoin_lightning_kinds(),
            content: SomeTestOrderParams::taker_obligation_bitcoin_rmb_content(),
        };
        let order1 = SomeTestOrderParams::default_builder()
            .trade_uuid(uuid1)
            .maker_obligation(maker_obligation1)
            .taker_obligation(taker_obligation1)
            .build()
            .unwrap();

        let maker_m1a_tester = MakerTester::start(manager_m1, order1, false).await;

        // Manager M2 posts Order 2 tracked by Maker M2A - USD for BTC/both
        let uuid2 = Uuid::new_v4();
        let maker_obligation2 = MakerObligation {
            kinds: SomeTestOrderParams::obligation_fiat_usd_kinds(),
            content: SomeTestOrderParams::maker_obligation_fiat_usd_content(),
        };
        let taker_obligation2 = TakerObligation {
            kinds: SomeTestOrderParams::obligation_bitcoin_both_kinds(),
            content: SomeTestOrderParams::taker_obligation_bitcoin_usd_content(),
        };
        let order2 = SomeTestOrderParams::default_builder()
            .trade_uuid(uuid2)
            .maker_obligation(maker_obligation2)
            .taker_obligation(taker_obligation2)
            .build()
            .unwrap();

        let maker_m2a_tester = MakerTester::start(manager_m2, order2, false).await;

        // Manager M3 posts Order 3 tracked by Maker M3A - BTC/onchain for EUR
        let uuid3 = Uuid::new_v4();
        let maker_obligation3 = MakerObligation {
            kinds: SomeTestOrderParams::obligation_bitcoin_onchain_kinds(),
            content: SomeTestOrderParams::maker_obligation_bitcoin_content(),
        };
        let taker_obligation3 = TakerObligation {
            kinds: SomeTestOrderParams::obligation_fiat_eur_kinds(),
            content: SomeTestOrderParams::taker_obligation_fiat_eur_content(),
        };
        let order3 = SomeTestOrderParams::default_builder()
            .trade_uuid(uuid3)
            .maker_obligation(maker_obligation3)
            .taker_obligation(taker_obligation3)
            .build()
            .unwrap();

        let maker_m3a_tester = MakerTester::start(manager_m3, order3, false).await;

        // Wait for all the relays to have the orders
        sleep(Duration::from_secs(1)).await;

        // Currently here are the Orders & Taking Managers ready for the test
        //
        // Order 1, Manager M1, Maker M1A, TradeEngine-1, RMB for BTC/LN,      Relays 1,2,3,4
        // Order 2, Manager M2, Maker M2A, TradeEngine-2, USD for BTC/both,    Relays 1,2,3
        // Order 3, Manager M3, Maker M3A, TradeEngine-1, BTC/onchain for EUR, Relay 1
        //
        // Manager T1 TradeEngine-1, Relays 1, 2, 4
        // Manager T2 TradeEngine-2, Relays 1, 3, 4

        // Taker 1 Filter for own TradeEngine-1 Orders,
        // Should get back
        // Order 1 from Relays 1, 2, 4
        // Order 3 from Relay 1

        let query_filter = Vec::new();
        let taker_tester = TakerTester::start(manager_t1, query_filter, None).await;
        let order_envelopes = taker_tester.wait_for_completion().await.unwrap();
        assert_eq!(order_envelopes.len(), 2);

        let order_envelopes_1 =
            SomeTestOrderParams::filter_for_trade_uuid(uuid1, order_envelopes.clone());
        assert_eq!(order_envelopes_1.len(), 1);
        assert!(order_envelopes_1
            .first()
            .unwrap()
            .urls
            .contains(&relay1_url));
        assert!(order_envelopes_1
            .first()
            .unwrap()
            .urls
            .contains(&relay2_url));
        assert!(order_envelopes_1
            .first()
            .unwrap()
            .urls
            .contains(&relay4_url));

        let order_envelopes_3 =
            SomeTestOrderParams::filter_for_trade_uuid(uuid3, order_envelopes.clone());
        assert_eq!(order_envelopes_3.len(), 1);
        assert!(order_envelopes_3
            .first()
            .unwrap()
            .urls
            .contains(&relay1_url));

        // Taker 2 Filter to buy BTC (Maker Obligation = BTC), should get nothing
        let mut query_filter = Vec::new();
        query_filter.push(FilterTag::MakerObligations(
            SomeTestOrderParams::obligation_bitcoin_both_kinds(),
        ));
        let taker_tester = TakerTester::start(manager_t2, query_filter, None).await;
        let order_envelopes = taker_tester.wait_for_completion().await.unwrap();
        assert_eq!(order_envelopes.len(), 0);

        maker_m1a_tester.wait_for_completion().await.unwrap();
        maker_m2a_tester.wait_for_completion().await.unwrap();
        maker_m3a_tester.wait_for_completion().await.unwrap();

        relay1.shutdown().unwrap();
        relay2.shutdown().unwrap();
        relay3.shutdown().unwrap();
        relay4.shutdown().unwrap();
    }
}
