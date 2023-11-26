use std::time::Duration;

use crusty_n3xb::{
    common::error::N3xbError,
    manager::Manager,
    offer::OfferEnvelope,
    order::Order,
    testing::{SomeTestOfferParams, SomeTestTradeRspParams, TESTING_DEFAULT_CHANNEL_SIZE},
};
use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

pub struct MakerTester {
    cmpl_rx: oneshot::Receiver<Result<(), N3xbError>>,
}

impl MakerTester {
    pub async fn start(manager: Manager, order: Order, wait_for_offer: bool) -> Self {
        let (cmpl_tx, cmpl_rx) = oneshot::channel::<Result<(), N3xbError>>();
        let actor = MakerTesterActor::new(cmpl_tx, manager, order, wait_for_offer).await;
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
    wait_for_offer: bool,
}

impl MakerTesterActor {
    async fn new(
        cmpl_tx: oneshot::Sender<Result<(), N3xbError>>,
        manager: Manager,
        order: Order,
        wait_for_offer: bool,
    ) -> Self {
        Self {
            cmpl_tx,
            manager,
            order,
            wait_for_offer,
        }
    }

    async fn run(self) {
        // Create and setup a Maker for a new Order
        let order = self.order.clone();
        let maker = self.manager.new_maker(order).await.unwrap();

        // Register Maker for Offer notificaitons
        let (notif_tx, mut notif_rx) =
            mpsc::channel::<Result<OfferEnvelope, N3xbError>>(TESTING_DEFAULT_CHANNEL_SIZE);
        maker.register_offer_notif_tx(notif_tx).await.unwrap();

        // The whole thing kicks off by sending a Maker Order Note
        let maker = maker.post_new_order().await.unwrap();

        if !self.wait_for_offer {
            sleep(Duration::from_secs(2)).await;
            maker.cancel_order().await.unwrap();
            self.manager.shutdown().await.unwrap();
            self.cmpl_tx.send(Ok(())).unwrap();
            return;
        }

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
        let maker = maker.accept_offer(trade_rsp).await.unwrap();

        maker.trade_complete().await.unwrap();
        self.manager.shutdown().await.unwrap();

        // Send Success Completion
        self.cmpl_tx.send(Ok(())).unwrap();
        // Thread Ends
    }
}
