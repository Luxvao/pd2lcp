use md5::{Digest, Md5};

use crate::error::Error;

pub fn compute_hash(data: &[u8]) -> Result<String, Error> {
    let mut hasher = Md5::new();
    hasher.update(data);

    Ok(hex::encode(hasher.finalize()))
}
