use super::super::Order;

pub struct TakerOrder {
    // Taker specific Order properties
}

impl TakerOrder {
    // Commands Takers can issue
    pub fn take() {}
}

impl Order for TakerOrder {
    fn identifier() -> String {
        String::new()
    }

    fn message() {}

    fn remove() {}

    fn complete() {}
}
