use super::super::Order;

pub struct TakerOrder {
    // Taker specific Order properties
}

impl TakerOrder {
    // Commands Takers can issue
    pub fn take() {}
}

impl Order for TakerOrder {
    fn identifier(&self) -> String {
        String::new()
    }

    fn message(&self) {}

    fn remove(&self) {}

    fn complete(&self) {}
}
