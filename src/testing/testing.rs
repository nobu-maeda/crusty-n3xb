pub struct SomeTestParams {}

impl SomeTestParams {
    pub fn engine_name_str() -> String {
        "some-trade-mechanics".to_string()
    }

    pub fn engine_specific_str() -> String {
        "some-test-specific-info".to_string()
    }
}

pub const TESTING_DEFAULT_CHANNEL_SIZE: usize = 5;
