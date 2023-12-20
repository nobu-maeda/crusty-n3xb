use std::path::Path;

use tokio::io::AsyncWriteExt;

use crate::common::error::N3xbError;

// TODO: Optional - Encrypt with private key before persisting data
pub async fn persist(json: String, path: impl AsRef<Path>) -> Result<(), N3xbError> {
    let path = path.as_ref();
    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(json.as_bytes()).await?;
    file.sync_all().await?;
    Ok(())
}

pub async fn restore(path: impl AsRef<Path>) -> Result<String, N3xbError> {
    let path = path.as_ref();
    let json = tokio::fs::read_to_string(path).await?;
    Ok(json)
}
