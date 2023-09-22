use std::net::SocketAddr;

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::common::error::N3xbError;
use crate::interfacer::{Interfacer, InterfacerHandle};
use crate::offer::Offer;
use crate::order::Order;
use crate::order_sm::maker::MakerSM;
use crate::order_sm::taker::TakerSM;

// At the moment we only support a single Trade Engine at a time.
// Might need to change to a dyn Trait if mulitple is to be supported at a time
pub struct Manager {
    trade_engine_name: String,
    interfacer: Interfacer,
    interfacer_handle: InterfacerHandle,
}

impl Manager {
    // Public Functions

    // Constructors

    // TODO: Should take in genericized Keys or Client, but also Trade Engine Specifics
    // TODO: Should also take in custom path for n3xB file locations

    pub async fn new(trade_engine_name: &str) -> Manager {
        let interfacer = Interfacer::new(trade_engine_name).await;
        let interfacer_handle = interfacer.new_handle();

        Manager {
            trade_engine_name: trade_engine_name.to_string(),
            interfacer,
            interfacer_handle,
        }
    }

    pub async fn new_with_keys(key: SecretKey, trade_engine_name: &str) -> Manager {
        let interfacer = Interfacer::new_with_key(key, trade_engine_name).await;
        let interfacer_handle = interfacer.new_handle();

        Manager {
            trade_engine_name: trade_engine_name.to_string(),
            interfacer,
            interfacer_handle,
        }
    }

    // Nostr Management
    pub async fn pubkey(&self) -> XOnlyPublicKey {
        self.interfacer_handle.get_public_key().await
    }

    pub async fn add_relays(
        &self,
        relays: Vec<(String, Option<SocketAddr>)>,
        connect: bool,
    ) -> Result<(), N3xbError> {
        self.interfacer_handle.add_relays(relays, connect).await?;
        Ok(())
    }

    // Order Management

    pub async fn make_new_order(&self, order: Order) -> Result<MakerSM, N3xbError> {
        let maker_sm = MakerSM::new(self.interfacer.new_handle(), order).await?;
        Ok(maker_sm)
    }

    pub async fn query_order_notes(&mut self) -> Result<Vec<Order>, N3xbError> {
        let orders = self.interfacer_handle.query_order_notes().await?;
        Ok(orders)
    }

    pub async fn take_order(&self, order: Order, offer: Offer) -> Result<TakerSM, N3xbError> {
        let taker_sm: TakerSM = TakerSM::new(self.interfacer.new_handle(), order, offer).await?;
        Ok(taker_sm)
    }

    fn load_settings() {
        // TODO: Read all files from relevant directories, scan for settings, and load into memory
        // Settings should be applied later as applicable from the memory location
    }

    // Private Functions
}
