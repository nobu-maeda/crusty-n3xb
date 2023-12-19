use std::str::FromStr;

use secp256k1::SecretKey;

pub struct SomeTestParams {}

impl SomeTestParams {
    pub fn engine_name_str() -> String {
        "some-trade-mechanics".to_string()
    }

    pub fn engine_specific_str() -> String {
        "some-test-specific-info".to_string()
    }

    pub fn maker_private_key() -> SecretKey {
        SecretKey::from_str("9709e361864037ef7b929c2b36dc36155568e9a066291dfadc79ed5d106e59f8")
            .unwrap()
    }

    pub fn taker_private_key() -> SecretKey {
        SecretKey::from_str("80e6f8e839135232972dfc16f2acdaeee9c0bcb4793a8a8249b7e384a51377e1")
            .unwrap()
    }
}

pub const TESTING_DEFAULT_CHANNEL_SIZE: usize = 5;
