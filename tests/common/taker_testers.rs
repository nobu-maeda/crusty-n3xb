use crusty_n3xb::{
    common::error::N3xbError,
    manager::Manager,
    order::{FilterTag, OrderEnvelope},
    testing::{
        SomeTestOfferParams, SomeTestOrderParams, SomeTestTradeRspParams,
        TESTING_DEFAULT_CHANNEL_SIZE,
    },
    trade_rsp::{TradeResponseEnvelope, TradeResponseStatus},
};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub struct TakerTester {
    cmpl_rx: oneshot::Receiver<Result<Vec<OrderEnvelope>, N3xbError>>,
}

impl TakerTester {
    pub async fn start(
        manager: Manager,
        query_filter: Vec<FilterTag>,
        take_trade_uuid: Option<Uuid>,
    ) -> Self {
        let (cmpl_tx, cmpl_rx) = oneshot::channel::<Result<Vec<OrderEnvelope>, N3xbError>>();
        let actor = TakerTesterActor::new(cmpl_tx, manager, query_filter, take_trade_uuid).await;
        tokio::spawn(async move { actor.run().await });
        Self { cmpl_rx }
    }

    pub async fn wait_for_completion(self) -> Result<Vec<OrderEnvelope>, N3xbError> {
        self.cmpl_rx.await.unwrap()
    }
}

struct TakerTesterActor {
    cmpl_tx: oneshot::Sender<Result<Vec<OrderEnvelope>, N3xbError>>,
    manager: Manager,
    query_filter: Vec<FilterTag>,
    take_trade_uuid: Option<Uuid>,
}

impl TakerTesterActor {
    async fn new(
        cmpl_tx: oneshot::Sender<Result<Vec<OrderEnvelope>, N3xbError>>,
        manager: Manager,
        query_filter: Vec<FilterTag>,
        take_trade_uuid: Option<Uuid>,
    ) -> Self {
        Self {
            cmpl_tx,
            manager,
            query_filter,
            take_trade_uuid,
        }
    }

    async fn run(self) {
        let timeout = tokio::time::Duration::from_secs(3);
        let start_time = tokio::time::Instant::now();

        let order_envelopes = loop {
            let order_envelopes = self
                .manager
                .query_orders(self.query_filter.clone())
                .await
                .unwrap();

            // If current time is greater than timeout, then shutdown, send completion and return
            if tokio::time::Instant::now() - start_time > timeout {
                self.manager.shutdown().await.unwrap();
                self.cmpl_tx.send(Ok(order_envelopes)).unwrap();
                return;
            } else if let Some(trade_uuid) = self.take_trade_uuid {
                let filtered_envelopes: Vec<OrderEnvelope> =
                    SomeTestOrderParams::filter_for_trade_uuid(trade_uuid, order_envelopes);
                if filtered_envelopes.len() > 0 {
                    break filtered_envelopes;
                }
            } else {
                if order_envelopes.len() > 0 {
                    self.manager.shutdown().await.unwrap();
                    self.cmpl_tx.send(Ok(order_envelopes)).unwrap();
                    return;
                }
            }
        };

        let order_envelope = order_envelopes.first().unwrap().clone();
        SomeTestOrderParams::check(
            &order_envelope.order,
            &SomeTestOrderParams::default_builder().build().unwrap(),
        );

        // Figure out which relay the Order is from

        // Create and setup a Taker for an Order with a new Offer
        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        let taker = self.manager.new_taker(order_envelope, offer).await.unwrap();

        // Register Taker for Trade Response notifications
        let (notif_tx, mut notif_rx) =
            mpsc::channel::<Result<TradeResponseEnvelope, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
        taker.register_trade_notif_tx(notif_tx).await.unwrap();

        // Take Order with configured Offer
        taker.take_order().await.unwrap();

        // Wait for Trade Response notifications
        let notif_result = notif_rx.recv().await.unwrap();
        let trade_rsp_envelope = notif_result.unwrap();

        let mut expected_trade_rsp_builder = SomeTestTradeRspParams::default_builder();
        expected_trade_rsp_builder.offer_event_id("".to_string());

        let expected_trade_rsp = expected_trade_rsp_builder.build().unwrap();
        SomeTestTradeRspParams::check(&trade_rsp_envelope.trade_rsp, &expected_trade_rsp);

        assert_eq!(
            trade_rsp_envelope.trade_rsp.trade_response,
            TradeResponseStatus::Accepted
        );

        taker.trade_complete().await.unwrap();
        self.manager.shutdown().await.unwrap();

        // Send Success Completion
        self.cmpl_tx.send(Ok(order_envelopes)).unwrap();

        // Thread Ends
    }
}
