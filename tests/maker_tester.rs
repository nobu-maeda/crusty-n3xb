use crusty_n3xb::{common::error::N3xbError, manager::Manager, offer::Offer, order::Order};
use tokio::sync::{mpsc, oneshot};

pub struct MakerTester {
    cmpl_rx: oneshot::Receiver<Result<(), N3xbError>>,
}

impl MakerTester {
    pub async fn start(
        manager: Manager,
        order: Order,
        offer_event_id: String,
        trade_rsp_event_id: String,
    ) -> Self {
        let (cmpl_tx, cmpl_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let actor =
            MakerTesterActor::new(cmpl_tx, manager, order, offer_event_id, trade_rsp_event_id)
                .await;
        tokio::spawn(async move { actor.run().await });
        Self { cmpl_rx }
    }

    pub async fn wait_for_completion(self) -> Result<(), N3xbError> {
        self.cmpl_rx.await.unwrap()
    }
}

struct MakerTesterActor {
    cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
    manager: Manager,
    order: Order,
    offer_event_id: String,
    trade_rsp_event_id: String,
}

impl MakerTesterActor {
    async fn new(
        cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
        manager: Manager,
        order: Order,
        offer_event_id: String,
        trade_rsp_event_id: String,
    ) -> Self {
        Self {
            cmpl_tx,
            manager,
            order,
            offer_event_id,
            trade_rsp_event_id,
        }
    }

    const MAKER_TEST_ACTOR_NOTIF_CHANNEL_SIZE: usize = 5;

    async fn run(self) {
        // The whole thing kicks off by sending a Maker Order
        let order = self.order.clone();
        let maker = self.manager.make_new_order(order).await.unwrap();

        // Register for Taker Offer Notifs from Maker
        let (notif_tx, mut notif_rx) =
            mpsc::channel::<Result<Offer, N3xbError>>(Self::MAKER_TEST_ACTOR_NOTIF_CHANNEL_SIZE);
        maker.register_offer_notif_tx(notif_tx).await.unwrap();

        // Wait for a Taker Offer Notif - This can be made into a loop if wanted, or to wait for a particular offer
        let notif_result = notif_rx.recv().await.unwrap();
        let offer = notif_result.unwrap();

        // Check that the Offer is as expected
        assert_eq!(offer.event_id, self.offer_event_id);

        // Query Offers
        let offers = maker.query_offers().await;
        assert!(offers.len() >= 1);
        assert!(offers.iter().any(|o| o.event_id == self.offer_event_id));

        // Accept Offer

        // Send Success Completion
        self.cmpl_tx.send(Ok(())).unwrap();
        // Thread Ends
    }
}
