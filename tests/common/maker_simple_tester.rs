use crusty_n3xb::{
    common::error::N3xbError,
    manager::Manager,
    offer::OfferEnvelope,
    order::Order,
    testing::{SomeTestOfferParams, SomeTestTradeRspParams},
};
use tokio::sync::{mpsc, oneshot};

pub struct MakerSimpleTester {
    cmpl_rx: oneshot::Receiver<Result<(), N3xbError>>,
}

impl MakerSimpleTester {
    pub async fn start(manager: Manager, order: Order) -> Self {
        let (cmpl_tx, cmpl_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let actor = MakerSimpleTesterActor::new(cmpl_tx, manager, order).await;
        tokio::spawn(async move { actor.run().await });
        Self { cmpl_rx }
    }

    pub async fn wait_for_completion(self) -> Result<(), N3xbError> {
        self.cmpl_rx.await.unwrap()
    }
}

struct MakerSimpleTesterActor {
    cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
    manager: Manager,
    order: Order,
}

impl MakerSimpleTesterActor {
    async fn new(
        cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
        manager: Manager,
        order: Order,
    ) -> Self {
        Self {
            cmpl_tx,
            manager,
            order,
        }
    }

    const MAKER_TEST_ACTOR_NOTIF_CHANNEL_SIZE: usize = 5;

    async fn run(self) {
        // Create and setup a Maker for a new Order
        let order = self.order.clone();
        let maker = self.manager.new_maker(order).await.unwrap();

        // Register Maker for Offer notificaitons
        let (notif_tx, mut notif_rx) = mpsc::channel::<Result<OfferEnvelope, N3xbError>>(
            Self::MAKER_TEST_ACTOR_NOTIF_CHANNEL_SIZE,
        );
        maker.register_offer_notif_tx(notif_tx).await.unwrap();

        // The whole thing kicks off by sending a Maker Order Note
        maker.post_new_order().await.unwrap();

        // Wait for Offer notifications - This can be made into a loop if wanted, or to wait for a particular offer
        let notif_result = notif_rx.recv().await.unwrap();
        let offer_envelope = notif_result.unwrap();

        // Query Offer
        let offer_envelopes = maker.query_offers().await;
        assert!(offer_envelopes.len() >= 1);

        let offer = maker
            .query_offer(offer_envelope.event_id.clone())
            .await
            .unwrap()
            .offer;
        offer.validate_against(&self.order).unwrap();

        SomeTestOfferParams::check(
            &offer,
            &SomeTestOfferParams::default_builder().build().unwrap(),
        );

        // Accept Offer
        let mut trade_rsp_builder = SomeTestTradeRspParams::default_builder();
        trade_rsp_builder.offer_event_id(offer_envelope.event_id);
        let trade_rsp = trade_rsp_builder.build().unwrap();
        maker.accept_offer(trade_rsp).await.unwrap();

        maker.trade_complete().await.unwrap();
        self.manager.shutdown().await.unwrap();

        // Send Success Completion
        self.cmpl_tx.send(Ok(())).unwrap();
        // Thread Ends
    }
}
