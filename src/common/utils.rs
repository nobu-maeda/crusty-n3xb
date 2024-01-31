use crate::common::error::N3xbError;
use std::{fs, path::Path};

// TODO: Optional - Encrypt with private key before persisting data
pub fn persist(json: String, path: impl AsRef<Path>) -> Result<(), N3xbError> {
    fs::write(path.as_ref(), json)?;
    Ok(())
}

pub fn restore(path: impl AsRef<Path>) -> Result<String, N3xbError> {
    let json = fs::read_to_string(path.as_ref())?;
    Ok(json)
}
