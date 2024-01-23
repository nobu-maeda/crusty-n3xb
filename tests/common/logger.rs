use std::sync::Once;

use log::LevelFilter;

static INIT: Once = Once::new();

// Setup function that is only run once, even if called multiple times
pub fn setup() {
    INIT.call_once(|| {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init()
    });
}
