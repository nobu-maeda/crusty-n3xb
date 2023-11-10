use crusty_n3xb::{
    common::error::N3xbError, manager::Manager, order::OrderEnvelope, testing::SomeTestOfferParams,
};
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct TakerTester {
    cmpl_rx: oneshot::Receiver<Result<(), N3xbError>>,
}

impl TakerTester {
    pub async fn start(manager: Manager, trade_uuid: Uuid) -> Self {
        let (cmpl_tx, cmpl_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let actor = TakerTesterActor::new(cmpl_tx, manager, trade_uuid).await;
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
}

impl TakerTesterActor {
    async fn new(
        cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
        manager: Manager,
        trade_uuid: Uuid,
    ) -> Self {
        Self {
            cmpl_tx,
            manager,
            trade_uuid,
        }
    }

    async fn run(mut self) {
        // Query & poll for Orders
        // * Optionally create ability to subscribe to a certain filter of Orders
        let order_envelope = loop {
            let order_envelopes = self.manager.query_orders().await.unwrap();
            let order_envelopes: Vec<OrderEnvelope> = order_envelopes
                .into_iter()
                .filter(|order_envelope| order_envelope.order.trade_uuid == self.trade_uuid)
                .collect();
            if order_envelopes.len() > 0 {
                break order_envelopes.first().unwrap().to_owned();
            }
        };

        // Take Order with Offer -> creates Taker
        let offer = SomeTestOfferParams::default_builder().build().unwrap();
        let taker = self
            .manager
            .take_order(order_envelope, offer)
            .await
            .unwrap();

        // Wait for Offer Acceptance Notif

        // Send Success Completion
        self.cmpl_tx.send(Ok(())).unwrap();

        // Thread Ends
    }
}
