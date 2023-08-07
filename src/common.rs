use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait SerdeGenericTrait: Serialize + DeserializeOwned + Clone + Debug {}
