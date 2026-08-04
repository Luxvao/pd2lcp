use std::io::Read;

use sha1::{Digest, Sha1};

use crate::error::Error;

pub fn compute_hash<R: Read>(mut reader: R) -> Result<String, Error> {
    let mut hasher = Sha1::new();

    loop {
        let mut buffer: Vec<u8> = Vec::new();

        let n = reader.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        hasher.update(buffer);
    }

    Ok(hex::encode(hasher.finalize()))
}
