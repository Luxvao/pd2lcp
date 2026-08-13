use std::path::Path;

use md5::{Digest, Md5};
use tokio::fs;

use crate::error::Error;

pub fn compute_hash(data: &[u8]) -> Result<String, Error> {
    let mut hasher = Md5::new();
    hasher.update(data);

    Ok(hex::encode(hasher.finalize()))
}

pub async fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst).await?;
    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let ty = entry.file_type().await?;
        let dst_path = dst.as_ref().join(entry.file_name());
        if ty.is_dir() {
            Box::pin(copy_dir_all(entry.path(), dst_path)).await?;
        } else {
            fs::copy(entry.path(), dst_path).await?;
        }
    }
    Ok(())
}
