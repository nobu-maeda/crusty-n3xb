use crusty_n3xb::{
    common::error::N3xbError, manager::Manager, order::Order, testing::SomeTestOfferParams,
};
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct TakerTester {
    cmpl_rx: oneshot::Receiver<Result<(), N3xbError>>,
}

impl TakerTester {
    pub async fn start(
        manager: Manager,
        trade_uuid: Uuid,
        offer_event_id: String,
        trade_rsp_event_id: String,
    ) -> Self {
        let (cmpl_tx, cmpl_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let actor = TakerTesterActor::new(
            cmpl_tx,
            manager,
            trade_uuid,
            offer_event_id,
            trade_rsp_event_id,
        )
        .await;
        tokio::spawn(async move { actor.run().await });
        Self { cmpl_rx }
    }

    pub async fn wait_for_completion(self) -> Result<(), N3xbError> {
        self.cmpl_rx.await.unwrap()
    }
}

struct TakerTesterActor {
    cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
    manager: Manager,
    trade_uuid: Uuid,
    offer_event_id: String,
    trade_rsp_event_id: String,
}

impl TakerTesterActor {
    async fn new(
        cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
        manager: Manager,
        trade_uuid: Uuid,
        offer_event_id: String,
        trade_rsp_event_id: String,
    ) -> Self {
        Self {
            cmpl_tx,
            manager,
            trade_uuid,
            offer_event_id,
            trade_rsp_event_id,
        }
    }

    async fn run(mut self) {
        // Query & poll for Orders
        // * Optionally create ability to subscribe to a certain filter of Orders
        let order = loop {
            let orders = self.manager.query_order_notes().await.unwrap();
            let orders: Vec<Order> = orders
                .into_iter()
                .filter(|order| order.trade_uuid == self.trade_uuid)
                .collect();
            if orders.len() > 0 {
                break orders.first().unwrap().to_owned();
            }
        };

        // Take Order with Offer -> creates Taker
        let mut builder = SomeTestOfferParams::default_builder();
        builder.event_id(self.offer_event_id); // This overrides the Event ID of the Taker Offer mostly for testing purposes. Normally this is generated as part of the ID of the Nostr Event.
        let offer = builder.build().unwrap();

        let taker = self.manager.take_order(order, offer).await.unwrap();

        // Wait for Offer Acceptance Notif

        // Send Success Completion
        self.cmpl_tx.send(Ok(())).unwrap();

        // Thread Ends
    }
}
