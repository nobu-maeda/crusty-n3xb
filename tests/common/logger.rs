use std::sync::Once;
use tracing_subscriber::prelude::*;

static INIT: Once = Once::new();

// Setup function that is only run once, even if called multiple times
pub fn setup() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    });
}
